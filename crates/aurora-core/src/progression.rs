//! Harmony progression mode and related enums shared across crates.

use serde::{Deserialize, Serialize};

/// How chord progressions are structured over a section.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressionMode {
    /// Repeating N-chord cell (e.g. 4-chord loop); seam quality scored at wrap.
    #[default]
    Loop,
    /// Phrase-directed arc without forced repetition; resolves at phrase/section ends.
    Flow,
}
