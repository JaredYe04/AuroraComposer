use serde::{Deserialize, Serialize};

/// Search-engine configuration derived from user parameters.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SearchConfig {
    pub beam_width: u16,
    pub temperature: f32,
    pub max_iterations: u32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            beam_width: 16,
            temperature: 1.0,
            max_iterations: 10_000,
        }
    }
}

impl SearchConfig {
    #[must_use]
    pub fn from_params(params: &crate::params::SearchParams) -> Self {
        Self {
            beam_width: params.beam_width,
            temperature: params.temperature,
            max_iterations: params.max_iterations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_beam_width_is_sixteen() {
        assert_eq!(SearchConfig::default().beam_width, 16);
    }

    #[test]
    fn from_params_overrides_defaults() {
        let params = crate::params::SearchParams {
            beam_width: 32,
            ..crate::params::SearchParams::default()
        };
        assert_eq!(SearchConfig::from_params(&params).beam_width, 32);
    }
}
