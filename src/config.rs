use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReassignmentBehavior {
    #[default]
    Allow,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub reassignment: ReassignmentBehavior,
    pub generate_fills: bool,
    // Add more pipeline flags here later
}

impl Default for Config {
    fn default() -> Self {
        Self {
            reassignment: ReassignmentBehavior::default(),
            generate_fills: true,
        }
    }
}
