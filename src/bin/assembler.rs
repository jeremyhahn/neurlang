//! Standalone assembler for Neurlang
//!
//! Converts text assembly to binary IR.

use anyhow::{Context, Result};
use clap::Parser;
use neurlang::ir::{Assembler, Disassembler};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nl-asm")]
#[command(about = "Neurlang Assembler")]
struct Args {
    /// Input file (use - for stdin)
    #[arg(default_value = "-")]
    input: String,

    /// Output file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Show disassembly
    #[arg(short, long)]
    disasm: bool,

    /// Output as hex instead of binary
    #[arg(long)]
    hex: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read input
    let source = if args.input == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        fs::read_to_string(&args.input).context("Failed to read input")?
    };

    // Assemble
    let mut asm = Assembler::new();
    let program = asm.assemble(&source).context("Assembly failed")?;

    eprintln!(
        "Assembled {} instructions ({} bytes)",
        program.instructions.len(),
        program.code_size()
    );

    // Show disassembly if requested
    if args.disasm {
        eprintln!("\nDisassembly:");
        let disasm = Disassembler::new().with_offsets(true);
        eprintln!("{}", disasm.disassemble(&program));
    }

    // Output
    let bytes = program.encode();

    if let Some(output) = args.output {
        if args.hex {
            let hex = hex::encode(&bytes);
            fs::write(&output, hex)?;
        } else {
            fs::write(&output, &bytes)?;
        }
        eprintln!("Wrote {} bytes to {}", bytes.len(), output.display());
    } else if args.hex {
        println!("{}", hex::encode(&bytes));
    } else {
        io::stdout().write_all(&bytes)?;
    }

    Ok(())
}
