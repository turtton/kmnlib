use destructure::Mutation;
use std::time::Duration;
use vodca::References;

#[derive(Debug, Clone, References, Mutation)]
pub struct MQConfig {
    worker_count: i32,
    max_retry: i32,
    retry_delay: Duration,
}

impl Default for MQConfig {
    fn default() -> Self {
        Self {
            worker_count: 4,
            max_retry: 3,
            retry_delay: Duration::from_secs(180),
        }
    }
}
