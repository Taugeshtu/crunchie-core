use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReassignmentBehavior {
    Allow,
    #[default]
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub reassignment: ReassignmentBehavior,
    pub generate_fills: bool,
    /// Pre-seeded constants. The Engine will strictly forbid assigning to these.
    pub constants: HashMap<String, f64>,
}

impl Default for Config {
    fn default() -> Self {
        let mut constants = HashMap::new();
        constants.insert("PI".to_string(), std::f64::consts::PI);
        constants.insert("TAU".to_string(), std::f64::consts::TAU);
        constants.insert("E".to_string(), std::f64::consts::E);

        Self {
            reassignment: ReassignmentBehavior::default(),
            generate_fills: true,
            constants,
        }
    }
}
