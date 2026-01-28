//! Dataset Loading for Neurlang Training
//!
//! Provides two dataset formats:
//! 1. Legacy: Intent + operand prediction (MultiHeadDataset)
//! 2. Parallel: Full instruction sequence prediction (ParallelDataset)
//!
//! The parallel format supports the new architecture that generates
//! up to 64 instructions in a single forward pass.

use burn::data::dataset::Dataset;
use burn::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::model::NUM_SLOTS;
use crate::inference::lookup::{detect_intent_from_keywords, OPERAND_COUNTS};
use crate::ir::Instruction;

/// Maximum sequence length
pub const MAX_SEQ_LEN: usize = 128;

/// Special tokens
pub const PAD_TOKEN: i64 = 256;
pub const UNK_TOKEN: i64 = 257;
pub const BOS_TOKEN: i64 = 258;
pub const EOS_TOKEN: i64 = 259;

// ============================================================================
// Parallel Instruction Dataset (NEW)
// ============================================================================

/// Raw sample for parallel training (JSONL format)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawParallelSample {
    /// Task description / prompt
    pub context: String,
    /// Partial IR built so far (for incremental generation)
    #[serde(default)]
    pub partial_ir: Vec<InstructionData>,
    /// Error feedback from previous attempt (for error correction training)
    #[serde(default)]
    pub error_feedback: Option<String>,
    /// Expected output instructions
    pub instructions: Vec<InstructionData>,
    /// Optional test cases for verification
    #[serde(default)]
    pub test_cases: Vec<TestCase>,
}

/// Single instruction data for training
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct InstructionData {
    /// Whether this slot is valid (1) or padding (0)
    #[serde(default)]
    pub valid: u8,
    /// Opcode index (0-32)
    pub opcode: u8,
    /// Mode bits (0-7)
    #[serde(default)]
    pub mode: u8,
    /// Destination register (0-31)
    #[serde(default)]
    pub rd: u8,
    /// Source register 1 (0-31)
    #[serde(default)]
    pub rs1: u8,
    /// Source register 2 (0-31)
    #[serde(default)]
    pub rs2: u8,
    /// Whether instruction has immediate
    #[serde(default)]
    pub has_imm: u8,
    /// Immediate value bin (0-255, quantized)
    #[serde(default)]
    pub imm_bin: u8,
}

/// Test case for verification
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestCase {
    /// Input values (register r0-r5)
    pub input: Vec<i64>,
    /// Expected output (r0)
    pub expected: i64,
}

/// Processed sample for parallel training
#[derive(Debug, Clone)]
pub struct ParallelSample {
    /// Tokenized input (length = MAX_SEQ_LEN)
    pub tokens: Vec<i64>,
    /// Target instructions (length = NUM_SLOTS)
    pub instructions: Vec<InstructionData>,
}

impl ParallelSample {
    /// Create a sample from raw data
    pub fn from_raw(raw: &RawParallelSample) -> Self {
        // Tokenize context (include error feedback if present)
        let mut text = raw.context.clone();
        if let Some(ref err) = raw.error_feedback {
            text.push_str(" [ERROR] ");
            text.push_str(err);
        }
        let tokens = tokenize(&text);

        // Pad instructions to NUM_SLOTS
        let mut instructions = raw.instructions.clone();
        // Mark all provided instructions as valid
        for instr in &mut instructions {
            instr.valid = 1;
        }
        // Pad with invalid (zero) instructions
        while instructions.len() < NUM_SLOTS {
            instructions.push(InstructionData::default());
        }
        instructions.truncate(NUM_SLOTS);

        Self {
            tokens,
            instructions,
        }
    }

    /// Create a sample from IR instructions
    pub fn from_instructions(context: &str, instrs: &[Instruction]) -> Self {
        let tokens = tokenize(context);

        let mut instructions: Vec<InstructionData> = instrs
            .iter()
            .take(NUM_SLOTS)
            .map(|i| InstructionData {
                valid: 1,
                opcode: i.opcode as u8,
                mode: i.mode,
                rd: i.rd as u8,
                rs1: i.rs1 as u8,
                rs2: i.rs2 as u8,
                has_imm: if i.imm.is_some() { 1 } else { 0 },
                imm_bin: quantize_immediate(i.imm.unwrap_or(0)),
            })
            .collect();

        // Pad with invalid instructions
        while instructions.len() < NUM_SLOTS {
            instructions.push(InstructionData::default());
        }

        Self {
            tokens,
            instructions,
        }
    }
}

/// Quantize immediate value to 8-bit bin
fn quantize_immediate(imm: i32) -> u8 {
    // For small values, direct mapping
    if imm >= 0 && imm < 128 {
        return imm as u8;
    }
    if imm < 0 && imm >= -128 {
        return (256 + imm) as u8;
    }
    // For larger values, use logarithmic binning
    let magnitude = (imm.abs() as f32).log2() as i32;
    let bin = (magnitude.min(15) + 128) as u8;
    if imm < 0 {
        bin | 0x80
    } else {
        bin
    }
}

/// Dequantize bin back to approximate immediate value
pub fn dequantize_immediate(bin: u8) -> i32 {
    if bin < 128 {
        return bin as i32;
    }
    if bin >= 128 && bin < 192 {
        // Positive large values
        let magnitude = (bin & 0x0F) as i32;
        return 1 << magnitude;
    }
    // Negative values
    if bin >= 192 {
        let magnitude = (bin & 0x0F) as i32;
        return -(1 << magnitude);
    }
    (bin as i32) - 256
}

/// Dataset for parallel instruction training
pub struct ParallelDataset {
    samples: Vec<ParallelSample>,
}

impl ParallelDataset {
    /// Load dataset from JSONL file
    pub fn from_jsonl<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let samples: Vec<ParallelSample> = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| serde_json::from_str::<RawParallelSample>(&line).ok())
            .map(|raw| ParallelSample::from_raw(&raw))
            .collect();

        Ok(Self { samples })
    }

    /// Create from existing IR samples (for backwards compatibility)
    pub fn from_legacy<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let samples: Vec<ParallelSample> = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| serde_json::from_str::<LegacyRawSample>(&line).ok())
            .filter_map(|raw| convert_legacy_to_parallel(&raw))
            .collect();

        Ok(Self { samples })
    }

    /// Create empty dataset
    pub fn empty() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    /// Get number of samples
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if dataset is empty
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Split dataset into train and validation sets
    pub fn split(self, train_ratio: f32) -> (Self, Self) {
        let split_idx = (self.samples.len() as f32 * train_ratio) as usize;
        let (train, val) = self.samples.split_at(split_idx);

        (
            Self {
                samples: train.to_vec(),
            },
            Self {
                samples: val.to_vec(),
            },
        )
    }

    /// Shuffle the dataset in-place
    pub fn shuffle(&mut self, seed: u64) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);

        let n = self.samples.len();
        for i in (1..n).rev() {
            hasher.write_usize(i);
            let j = (hasher.finish() as usize) % (i + 1);
            self.samples.swap(i, j);
        }
    }
}

impl Dataset<ParallelSample> for ParallelDataset {
    fn get(&self, index: usize) -> Option<ParallelSample> {
        self.samples.get(index).cloned()
    }

    fn len(&self) -> usize {
        self.samples.len()
    }
}

/// Batched training data for parallel model
#[derive(Debug, Clone)]
pub struct ParallelBatch<B: Backend> {
    /// Input tokens (batch, seq_len)
    pub tokens: Tensor<B, 2, Int>,
    /// Valid flags (batch, 64)
    pub valid: Tensor<B, 2, Int>,
    /// Opcode labels (batch, 64)
    pub opcode: Tensor<B, 2, Int>,
    /// Mode labels (batch, 64)
    pub mode: Tensor<B, 2, Int>,
    /// Destination register labels (batch, 64)
    pub rd: Tensor<B, 2, Int>,
    /// Source register 1 labels (batch, 64)
    pub rs1: Tensor<B, 2, Int>,
    /// Source register 2 labels (batch, 64)
    pub rs2: Tensor<B, 2, Int>,
    /// Has immediate flags (batch, 64)
    pub has_imm: Tensor<B, 2, Int>,
    /// Immediate bin labels (batch, 64)
    pub imm: Tensor<B, 2, Int>,
}

/// Batcher for parallel dataset
#[derive(Clone)]
pub struct ParallelBatcher;

impl ParallelBatcher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ParallelBatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Backend> burn::data::dataloader::batcher::Batcher<B, ParallelSample, ParallelBatch<B>>
    for ParallelBatcher
{
    fn batch(&self, items: Vec<ParallelSample>, device: &B::Device) -> ParallelBatch<B> {
        let batch_size = items.len();

        // Collect tokens
        let tokens: Vec<i64> = items
            .iter()
            .flat_map(|s| s.tokens.iter().copied())
            .collect();
        let tokens = Tensor::<B, 1, Int>::from_data(tokens.as_slice(), device)
            .reshape([batch_size, MAX_SEQ_LEN]);

        // Collect instruction fields
        let mut valid_vec = Vec::with_capacity(batch_size * NUM_SLOTS);
        let mut opcode_vec = Vec::with_capacity(batch_size * NUM_SLOTS);
        let mut mode_vec = Vec::with_capacity(batch_size * NUM_SLOTS);
        let mut rd_vec = Vec::with_capacity(batch_size * NUM_SLOTS);
        let mut rs1_vec = Vec::with_capacity(batch_size * NUM_SLOTS);
        let mut rs2_vec = Vec::with_capacity(batch_size * NUM_SLOTS);
        let mut has_imm_vec = Vec::with_capacity(batch_size * NUM_SLOTS);
        let mut imm_vec = Vec::with_capacity(batch_size * NUM_SLOTS);

        for sample in &items {
            for instr in &sample.instructions {
                valid_vec.push(instr.valid as i64);
                opcode_vec.push(instr.opcode as i64);
                mode_vec.push(instr.mode as i64);
                rd_vec.push(instr.rd as i64);
                rs1_vec.push(instr.rs1 as i64);
                rs2_vec.push(instr.rs2 as i64);
                has_imm_vec.push(instr.has_imm as i64);
                imm_vec.push(instr.imm_bin as i64);
            }
        }

        ParallelBatch {
            tokens,
            valid: Tensor::<B, 1, Int>::from_data(valid_vec.as_slice(), device)
                .reshape([batch_size, NUM_SLOTS]),
            opcode: Tensor::<B, 1, Int>::from_data(opcode_vec.as_slice(), device)
                .reshape([batch_size, NUM_SLOTS]),
            mode: Tensor::<B, 1, Int>::from_data(mode_vec.as_slice(), device)
                .reshape([batch_size, NUM_SLOTS]),
            rd: Tensor::<B, 1, Int>::from_data(rd_vec.as_slice(), device)
                .reshape([batch_size, NUM_SLOTS]),
            rs1: Tensor::<B, 1, Int>::from_data(rs1_vec.as_slice(), device)
                .reshape([batch_size, NUM_SLOTS]),
            rs2: Tensor::<B, 1, Int>::from_data(rs2_vec.as_slice(), device)
                .reshape([batch_size, NUM_SLOTS]),
            has_imm: Tensor::<B, 1, Int>::from_data(has_imm_vec.as_slice(), device)
                .reshape([batch_size, NUM_SLOTS]),
            imm: Tensor::<B, 1, Int>::from_data(imm_vec.as_slice(), device)
                .reshape([batch_size, NUM_SLOTS]),
        }
    }
}

// ============================================================================
// Legacy Dataset (backwards compatibility)
// ============================================================================

/// Legacy raw sample from JSONL file (matches old datagen output format)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LegacyRawSample {
    /// Input text (prompt)
    pub prompt: String,
    /// High-level category (e.g., "crypto", "arithmetic")
    pub category: String,
    /// Binary IR (optional)
    #[serde(default)]
    pub binary_ir: Vec<u8>,
    /// Assembly code (optional)
    #[serde(default)]
    pub assembly: Option<String>,
    /// Expected output (optional)
    #[serde(default)]
    pub expected_output: Option<i64>,
    /// Curriculum level
    #[serde(default)]
    pub level: u8,
}

// Re-export as RawSample for backwards compatibility
pub type RawSample = LegacyRawSample;

/// Convert legacy sample to parallel format
fn convert_legacy_to_parallel(raw: &LegacyRawSample) -> Option<ParallelSample> {
    // Parse binary IR if available
    if !raw.binary_ir.is_empty() {
        let mut instructions = Vec::new();
        let mut offset = 0;
        while offset < raw.binary_ir.len() {
            if let Some(instr) = Instruction::decode(&raw.binary_ir[offset..]) {
                let size = instr.size();
                instructions.push(InstructionData {
                    valid: 1,
                    opcode: instr.opcode as u8,
                    mode: instr.mode,
                    rd: instr.rd as u8,
                    rs1: instr.rs1 as u8,
                    rs2: instr.rs2 as u8,
                    has_imm: if instr.imm.is_some() { 1 } else { 0 },
                    imm_bin: quantize_immediate(instr.imm.unwrap_or(0)),
                });
                offset += size;
            } else {
                break;
            }
        }

        if !instructions.is_empty() {
            return Some(ParallelSample {
                tokens: tokenize(&raw.prompt),
                instructions: {
                    let mut padded = instructions;
                    while padded.len() < NUM_SLOTS {
                        padded.push(InstructionData::default());
                    }
                    padded.truncate(NUM_SLOTS);
                    padded
                },
            });
        }
    }

    None
}

/// Processed sample for legacy training
#[derive(Debug, Clone)]
pub struct MultiHeadSample {
    /// Tokenized input (length = MAX_SEQ_LEN)
    pub tokens: Vec<i64>,
    /// Intent class (0-53)
    pub intent: usize,
    /// Operand count (0-4)
    pub count: usize,
    /// Binned operand values (mod 256)
    pub operand_bins: [usize; 4],
    /// Signs for each operand (0=positive, 1=negative)
    pub signs: [usize; 4],
}

impl MultiHeadSample {
    /// Create a sample from raw data
    pub fn from_raw(raw: &LegacyRawSample) -> Option<Self> {
        let tokens = tokenize(&raw.prompt);

        // Detect intent from prompt using keyword matching
        let intent = match detect_intent_from_keywords(&raw.prompt) {
            Some((intent_id, _confidence)) => intent_id,
            None => return None,
        };

        // Extract operands from the prompt
        let operands = extract_numbers(&raw.prompt);
        let expected_operand_count = OPERAND_COUNTS.get(intent).copied().unwrap_or(0);
        let operand_count = operands.len().min(expected_operand_count).min(4);

        let mut operand_bins = [0usize; 4];
        let mut signs = [0usize; 4];

        for (i, &op) in operands.iter().take(4).enumerate() {
            operand_bins[i] = (op.unsigned_abs() % 256) as usize;
            signs[i] = if op < 0 { 1 } else { 0 };
        }

        Some(Self {
            tokens,
            intent,
            count: operand_count,
            operand_bins,
            signs,
        })
    }
}

/// Extract numbers from text
fn extract_numbers(text: &str) -> Vec<i64> {
    let mut numbers = Vec::new();
    let mut current_num = String::new();
    let mut is_negative = false;

    for c in text.chars() {
        if c == '-' && current_num.is_empty() {
            is_negative = true;
        } else if c.is_ascii_digit() {
            current_num.push(c);
        } else if !current_num.is_empty() {
            if let Ok(n) = current_num.parse::<i64>() {
                numbers.push(if is_negative { -n } else { n });
            }
            current_num.clear();
            is_negative = false;
        } else {
            is_negative = false;
        }
    }

    if !current_num.is_empty() {
        if let Ok(n) = current_num.parse::<i64>() {
            numbers.push(if is_negative { -n } else { n });
        }
    }

    numbers
}

/// Tokenize text to byte-level tokens with special tokens
pub fn tokenize(text: &str) -> Vec<i64> {
    let mut tokens = Vec::with_capacity(MAX_SEQ_LEN);

    // Add BOS token
    tokens.push(BOS_TOKEN);

    // Add byte tokens (leave room for EOS)
    for byte in text.bytes().take(MAX_SEQ_LEN - 2) {
        tokens.push(byte as i64);
    }

    // Add EOS token
    tokens.push(EOS_TOKEN);

    // Pad to MAX_SEQ_LEN
    while tokens.len() < MAX_SEQ_LEN {
        tokens.push(PAD_TOKEN);
    }

    tokens
}

/// Dataset for legacy multi-head training
pub struct MultiHeadDataset {
    samples: Vec<MultiHeadSample>,
}

impl MultiHeadDataset {
    /// Load dataset from JSONL file
    pub fn from_jsonl<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let samples: Vec<MultiHeadSample> = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| serde_json::from_str::<LegacyRawSample>(&line).ok())
            .filter_map(|raw| MultiHeadSample::from_raw(&raw))
            .collect();

        Ok(Self { samples })
    }

    /// Create empty dataset
    pub fn empty() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    /// Get number of samples
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if dataset is empty
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Split dataset into train and validation sets
    pub fn split(self, train_ratio: f32) -> (Self, Self) {
        let split_idx = (self.samples.len() as f32 * train_ratio) as usize;
        let (train, val) = self.samples.split_at(split_idx);

        (
            Self {
                samples: train.to_vec(),
            },
            Self {
                samples: val.to_vec(),
            },
        )
    }

    /// Shuffle the dataset in-place using a random seed
    pub fn shuffle(&mut self, seed: u64) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);

        let n = self.samples.len();
        for i in (1..n).rev() {
            hasher.write_usize(i);
            let j = (hasher.finish() as usize) % (i + 1);
            self.samples.swap(i, j);
        }
    }
}

impl Dataset<MultiHeadSample> for MultiHeadDataset {
    fn get(&self, index: usize) -> Option<MultiHeadSample> {
        self.samples.get(index).cloned()
    }

    fn len(&self) -> usize {
        self.samples.len()
    }
}

/// Batched training data for legacy model
#[derive(Debug, Clone)]
pub struct MultiHeadBatch<B: Backend> {
    /// Input tokens (batch, seq_len)
    pub tokens: Tensor<B, 2, Int>,
    /// Intent labels (batch,)
    pub intent: Tensor<B, 1, Int>,
    /// Count labels (batch,)
    pub count: Tensor<B, 1, Int>,
    /// Operand labels (batch, 4)
    pub operands: Tensor<B, 2, Int>,
    /// Sign labels (batch, 4)
    pub signs: Tensor<B, 2, Int>,
}

/// Batcher for multi-head dataset
#[derive(Clone)]
pub struct MultiHeadBatcher;

impl MultiHeadBatcher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MultiHeadBatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Backend> burn::data::dataloader::batcher::Batcher<B, MultiHeadSample, MultiHeadBatch<B>>
    for MultiHeadBatcher
{
    fn batch(&self, items: Vec<MultiHeadSample>, device: &B::Device) -> MultiHeadBatch<B> {
        let batch_size = items.len();

        let tokens: Vec<i64> = items
            .iter()
            .flat_map(|s| s.tokens.iter().copied())
            .collect();
        let tokens = Tensor::<B, 1, Int>::from_data(tokens.as_slice(), device)
            .reshape([batch_size, MAX_SEQ_LEN]);

        let intent: Vec<i64> = items.iter().map(|s| s.intent as i64).collect();
        let intent = Tensor::<B, 1, Int>::from_data(intent.as_slice(), device);

        let count: Vec<i64> = items.iter().map(|s| s.count as i64).collect();
        let count = Tensor::<B, 1, Int>::from_data(count.as_slice(), device);

        let operands: Vec<i64> = items
            .iter()
            .flat_map(|s| s.operand_bins.iter().map(|&b| b as i64))
            .collect();
        let operands =
            Tensor::<B, 1, Int>::from_data(operands.as_slice(), device).reshape([batch_size, 4]);

        let signs: Vec<i64> = items
            .iter()
            .flat_map(|s| s.signs.iter().map(|&b| b as i64))
            .collect();
        let signs =
            Tensor::<B, 1, Int>::from_data(signs.as_slice(), device).reshape([batch_size, 4]);

        MultiHeadBatch {
            tokens,
            intent,
            count,
            operands,
            signs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("hello");
        assert_eq!(tokens.len(), MAX_SEQ_LEN);
        assert_eq!(tokens[0], BOS_TOKEN);
        assert_eq!(tokens[1], b'h' as i64);
        assert_eq!(tokens[6], EOS_TOKEN);
        assert_eq!(tokens[7], PAD_TOKEN);
    }

    #[test]
    fn test_legacy_sample_from_raw() {
        let raw = LegacyRawSample {
            prompt: "add 5 and 10".to_string(),
            category: "arithmetic".to_string(),
            binary_ir: vec![],
            assembly: None,
            expected_output: None,
            level: 1,
        };

        let sample = MultiHeadSample::from_raw(&raw).expect("Should parse successfully");
        assert_eq!(sample.intent, 0); // ADD
        assert_eq!(sample.count, 2);
        assert_eq!(sample.operand_bins[0], 5);
        assert_eq!(sample.operand_bins[1], 10);
        assert_eq!(sample.signs[0], 0);
        assert_eq!(sample.signs[1], 0);
    }

    #[test]
    fn test_extract_numbers() {
        assert_eq!(extract_numbers("add 5 and 10"), vec![5, 10]);
        assert_eq!(extract_numbers("factorial of 7"), vec![7]);
        assert_eq!(extract_numbers("gcd(66, 79)"), vec![66, 79]);
        assert_eq!(extract_numbers("compute -42 + 100"), vec![-42, 100]);
    }

    #[test]
    fn test_quantize_dequantize() {
        // Small positive values
        assert_eq!(quantize_immediate(0), 0);
        assert_eq!(quantize_immediate(42), 42);
        assert_eq!(quantize_immediate(127), 127);
        assert_eq!(dequantize_immediate(42), 42);

        // Small negative values
        let neg5 = quantize_immediate(-5);
        assert!(neg5 >= 128);
    }

    #[test]
    fn test_parallel_sample() {
        let raw = RawParallelSample {
            context: "add two numbers".to_string(),
            partial_ir: vec![],
            error_feedback: None,
            instructions: vec![
                InstructionData {
                    valid: 1,
                    opcode: 0x1C, // MOV
                    mode: 0,
                    rd: 0,
                    rs1: 31,
                    rs2: 0,
                    has_imm: 1,
                    imm_bin: 5,
                },
                InstructionData {
                    valid: 1,
                    opcode: 0x00, // ALU
                    mode: 0,      // ADD
                    rd: 0,
                    rs1: 0,
                    rs2: 1,
                    has_imm: 0,
                    imm_bin: 0,
                },
            ],
            test_cases: vec![],
        };

        let sample = ParallelSample::from_raw(&raw);
        assert_eq!(sample.tokens.len(), MAX_SEQ_LEN);
        assert_eq!(sample.instructions.len(), NUM_SLOTS);
        assert_eq!(sample.instructions[0].valid, 1);
        assert_eq!(sample.instructions[0].opcode, 0x1C);
        assert_eq!(sample.instructions[1].valid, 1);
        assert_eq!(sample.instructions[2].valid, 0); // Padding
    }
}
