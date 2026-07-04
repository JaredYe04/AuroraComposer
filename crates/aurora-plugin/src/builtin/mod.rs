//! Built-in T0 style plugins shipped with Aurora Composer.

mod classical;
mod jazz;
mod pop;

pub use classical::ClassicalStylePlugin;
pub use jazz::JazzStylePlugin;
pub use pop::PopStylePlugin;
