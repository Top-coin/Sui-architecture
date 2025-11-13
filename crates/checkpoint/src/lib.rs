use sui_core::messages::CheckpointSummary;

#[derive(Default)]
pub struct CheckpointAggregator {
    summaries: Vec<CheckpointSummary>,
}

impl CheckpointAggregator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, summary: CheckpointSummary) {
        self.summaries.push(summary);
    }

    pub fn latest(&self) -> Option<&CheckpointSummary> {
        self.summaries.last()
    }

    pub fn total_transactions(&self) -> usize {
        self.summaries.iter().map(|s| s.transaction_count).sum()
    }
}

