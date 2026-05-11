use fend_core::Context;
use std::collections::HashSet;

pub struct Engine {
    pub ctx: Context,
}

impl Engine {
    pub fn bootstrap() -> Self {
        let ctx = Context::new();
        Self { ctx }
    }

    pub fn get_unit_names(&self) -> HashSet<String> {
        // Fend doesn't expose a simple list of unit names in the same way Numbat does.
        // For now, we'll return an empty set or a common subset.
        // In the future, we might want to hardcode common units or find a way to query Fend.
        HashSet::new()
    }
}
