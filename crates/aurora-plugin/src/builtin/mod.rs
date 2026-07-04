//! Built-in T0 style plugins shipped with Aurora Composer.

mod classical;
mod jazz;
mod pop;
mod ai_stub;

pub use classical::ClassicalStylePlugin;
pub use jazz::JazzStylePlugin;
pub use pop::PopStylePlugin;
pub use ai_stub::AiStubPlugin;
