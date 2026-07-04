use aurora_ast::{blank_workbench, Composition};
use aurora_core::{ParameterBundle, UiParameterSnapshot};
use aurora_plugin::PluginHost;
use std::sync::Mutex;

pub struct AppState {
    pub parameters: Mutex<ParameterBundle>,
    pub composition: Mutex<Option<Composition>>,
    pub plugin_host: Mutex<PluginHost>,
}

impl Default for AppState {
    fn default() -> Self {
        let params = ParameterBundle::default();
        let ui = UiParameterSnapshot::from(&params);
        let blank = blank_workbench("Untitled", ui.bars.max(1), ui.tempo_bpm, ui.key);
        Self {
            parameters: Mutex::new(params),
            composition: Mutex::new(Some(blank)),
            plugin_host: Mutex::new(PluginHost::new()),
        }
    }
}
