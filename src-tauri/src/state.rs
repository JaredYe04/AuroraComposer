use aurora_ast::Composition;
use aurora_core::ParameterBundle;
use aurora_plugin::PluginHost;
use std::sync::Mutex;

pub struct AppState {
    pub parameters: Mutex<ParameterBundle>,
    pub composition: Mutex<Option<Composition>>,
    pub plugin_host: Mutex<PluginHost>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            parameters: Mutex::new(ParameterBundle::default()),
            composition: Mutex::new(None),
            plugin_host: Mutex::new(PluginHost::new()),
        }
    }
}
