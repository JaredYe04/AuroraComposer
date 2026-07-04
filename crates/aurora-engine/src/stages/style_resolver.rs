use aurora_core::ParameterBundle;
use aurora_plugin::{PluginHost, StyleResolveRequest};

/// Active rule bundles and style metadata from StylePlugin resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedStyle {
    pub genre: String,
    pub era: String,
    pub active_bundles: Vec<String>,
    pub jazz_harmony: bool,
    pub plugin_id: String,
    pub parameters: ParameterBundle,
}

/// Stage 1 — Style Resolver: genre → parameter expansion + rule bundle switch via StylePlugins.
pub fn resolve_style(params: &ParameterBundle) -> ResolvedStyle {
    resolve_style_with_host(params, &PluginHost::new())
}

pub fn resolve_style_with_host(params: &ParameterBundle, host: &PluginHost) -> ResolvedStyle {
    let genre = params.style.genre.clone();
    let request = StyleResolveRequest {
        preset_id: genre.to_lowercase(),
        user_overrides: params.clone(),
    };

    let plugin_id = match genre.to_lowercase().as_str() {
        "jazz" | "blues" | "fusion" | "swing" | "bebop" => "com.aurora.plugins.jazz-style",
        "classical" | "baroque" | "romantic" | "chamber" => "com.aurora.plugins.classical-style",
        _ => "com.aurora.plugins.pop-style",
    };

    let resolved = host
        .resolve_style(plugin_id, &request)
        .unwrap_or_else(|_| fallback_resolve(params));

    ResolvedStyle {
        genre: resolved.parameters.style.genre.clone(),
        era: resolved.parameters.style.era.clone(),
        active_bundles: resolved.active_bundles,
        jazz_harmony: resolved.jazz_harmony,
        plugin_id: plugin_id.into(),
        parameters: resolved.parameters,
    }
}

fn fallback_resolve(params: &ParameterBundle) -> aurora_plugin::StyleResolveResult {
    let jazz_harmony = matches!(
        params.style.genre.to_lowercase().as_str(),
        "jazz" | "blues" | "fusion" | "swing" | "bebop"
    );
    let active_bundles = if jazz_harmony {
        vec!["JAZZ-*".into(), "HARM-*".into(), "VL-*".into()]
    } else {
        vec!["HARM-*".into(), "RHY-*".into()]
    };
    aurora_plugin::StyleResolveResult {
        parameters: params.clone(),
        active_plugins: Vec::new(),
        active_bundles,
        jazz_harmony,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::ParameterBundle;

    #[test]
    fn jazz_genre_enables_jazz_bundle() {
        let mut params = ParameterBundle::default();
        params.style.genre = "jazz".into();
        let resolved = resolve_style(&params);
        assert!(resolved.jazz_harmony);
        assert!(resolved.active_bundles.iter().any(|b| b.starts_with("JAZZ")));
        assert_eq!(resolved.plugin_id, "com.aurora.plugins.jazz-style");
    }

    #[test]
    fn classical_genre_uses_classical_plugin() {
        let mut params = ParameterBundle::default();
        params.style.genre = "classical".into();
        let resolved = resolve_style(&params);
        assert!(!resolved.jazz_harmony);
        assert!(resolved.active_bundles.contains(&"HARM-*".into()));
        assert_eq!(resolved.plugin_id, "com.aurora.plugins.classical-style");
    }

    #[test]
    fn pop_defaults_to_pop_plugin() {
        let params = ParameterBundle::default();
        let resolved = resolve_style(&params);
        assert_eq!(resolved.plugin_id, "com.aurora.plugins.pop-style");
    }
}
