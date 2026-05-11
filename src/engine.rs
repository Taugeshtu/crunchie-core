use numbat::{Context, resolver::CodeSource, module_importer::BuiltinModuleImporter};
use std::collections::HashSet;

pub struct Engine {
    pub ctx: Context,
}

impl Engine {
    pub fn bootstrap() -> Self {
        let mut ctx = Context::new(BuiltinModuleImporter::default());
        
        // Load the SI units prelude
        let result = ctx.interpret("use units::si", CodeSource::Internal);
        if let Err(e) = result {
            eprintln!("Failed to load SI units: {:?}", e);
        }
        
        Self { ctx }
    }

    pub fn get_unit_names(&self) -> HashSet<String> {
        let mut units = HashSet::new();
        for unit_group in self.ctx.unit_names() {
            for alias in unit_group {
                units.insert(alias.to_string());
            }
        }
        units
    }
}
