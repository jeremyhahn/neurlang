//! Instruction coverage tracking for Neurlang programs.
//!
//! When enabled, tracks which instructions are executed during program runs.
//! Useful for test coverage analysis.

use std::collections::{HashMap, HashSet};

/// Tracks which instructions were executed during program execution.
#[derive(Debug, Default)]
pub struct CoverageTracker {
    /// Set of executed program counter values
    executed_pcs: HashSet<usize>,

    /// Execution count per PC (for hot path analysis)
    execution_counts: HashMap<usize, u64>,

    /// Branch outcomes: (source_pc) -> (taken_count, not_taken_count)
    branch_outcomes: HashMap<usize, (u64, u64)>,

    /// Total instruction count
    total_instructions: usize,

    /// Whether tracking is enabled
    enabled: bool,
}

impl CoverageTracker {
    /// Create a new coverage tracker
    pub fn new(total_instructions: usize) -> Self {
        Self {
            executed_pcs: HashSet::with_capacity(total_instructions),
            execution_counts: HashMap::with_capacity(total_instructions),
            branch_outcomes: HashMap::new(),
            total_instructions,
            enabled: true,
        }
    }

    /// Create a disabled tracker (no-op)
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Mark an instruction as executed
    #[inline]
    pub fn mark_executed(&mut self, pc: usize) {
        if !self.enabled {
            return;
        }
        self.executed_pcs.insert(pc);
        *self.execution_counts.entry(pc).or_insert(0) += 1;
    }

    /// Mark a branch outcome
    #[inline]
    pub fn mark_branch(&mut self, pc: usize, taken: bool) {
        if !self.enabled {
            return;
        }
        let entry = self.branch_outcomes.entry(pc).or_insert((0, 0));
        if taken {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }

    /// Get instruction coverage percentage
    pub fn instruction_coverage(&self) -> f64 {
        if self.total_instructions == 0 {
            return 100.0;
        }
        (self.executed_pcs.len() as f64 / self.total_instructions as f64) * 100.0
    }

    /// Get branch coverage percentage
    pub fn branch_coverage(&self) -> f64 {
        if self.branch_outcomes.is_empty() {
            return 100.0;
        }

        let mut covered = 0;
        let mut total = 0;

        for (taken, not_taken) in self.branch_outcomes.values() {
            total += 2; // Each branch has two outcomes
            if *taken > 0 {
                covered += 1;
            }
            if *not_taken > 0 {
                covered += 1;
            }
        }

        (covered as f64 / total as f64) * 100.0
    }

    /// Get count of executed instructions
    pub fn executed_count(&self) -> usize {
        self.executed_pcs.len()
    }

    /// Get set of executed PCs
    pub fn executed_pcs(&self) -> &HashSet<usize> {
        &self.executed_pcs
    }

    /// Get uncovered instruction indices
    pub fn uncovered_pcs(&self) -> Vec<usize> {
        (0..self.total_instructions)
            .filter(|pc| !self.executed_pcs.contains(pc))
            .collect()
    }

    /// Get hot paths (most executed instructions)
    pub fn hot_paths(&self, top_n: usize) -> Vec<(usize, u64)> {
        let mut counts: Vec<_> = self
            .execution_counts
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1));
        counts.truncate(top_n);
        counts
    }

    /// Generate a coverage report
    pub fn report(&self) -> CoverageReport {
        CoverageReport {
            total_instructions: self.total_instructions,
            executed_instructions: self.executed_pcs.len(),
            instruction_coverage: self.instruction_coverage(),
            branch_coverage: self.branch_coverage(),
            uncovered: self.uncovered_pcs(),
        }
    }
}

/// Coverage report data
#[derive(Debug)]
pub struct CoverageReport {
    pub total_instructions: usize,
    pub executed_instructions: usize,
    pub instruction_coverage: f64,
    pub branch_coverage: f64,
    pub uncovered: Vec<usize>,
}

impl std::fmt::Display for CoverageReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Coverage Report")?;
        writeln!(f, "===============")?;
        writeln!(
            f,
            "Instructions: {}/{} ({:.1}%)",
            self.executed_instructions, self.total_instructions, self.instruction_coverage
        )?;
        writeln!(f, "Branches: {:.1}%", self.branch_coverage)?;

        if !self.uncovered.is_empty() && self.uncovered.len() <= 10 {
            writeln!(f, "\nUncovered instructions: {:?}", self.uncovered)?;
        } else if !self.uncovered.is_empty() {
            writeln!(
                f,
                "\nUncovered: {} instructions (first 10: {:?}...)",
                self.uncovered.len(),
                &self.uncovered[..10.min(self.uncovered.len())]
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_coverage() {
        let mut tracker = CoverageTracker::new(10);

        tracker.mark_executed(0);
        tracker.mark_executed(1);
        tracker.mark_executed(2);

        assert_eq!(tracker.executed_count(), 3);
        assert_eq!(tracker.instruction_coverage(), 30.0);
    }

    #[test]
    fn test_branch_coverage() {
        let mut tracker = CoverageTracker::new(10);

        // Branch at PC 5: taken once, not taken once
        tracker.mark_branch(5, true);
        tracker.mark_branch(5, false);

        // Branch at PC 7: only taken
        tracker.mark_branch(7, true);

        // 3 out of 4 outcomes covered
        assert_eq!(tracker.branch_coverage(), 75.0);
    }

    #[test]
    fn test_disabled_tracker() {
        let mut tracker = CoverageTracker::disabled();
        tracker.mark_executed(0);
        tracker.mark_executed(1);

        assert_eq!(tracker.executed_count(), 0);
    }
}
