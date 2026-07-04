use aurora_ast::Composition;
use aurora_core::ParameterBundle;
use std::sync::Mutex;

pub struct AppState {
    pub parameters: Mutex<ParameterBundle>,
    pub composition: Mutex<Option<Composition>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            parameters: Mutex::new(ParameterBundle::default()),
            composition: Mutex::new(None),
        }
    }
}
