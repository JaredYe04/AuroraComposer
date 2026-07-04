//! Copy-on-write AST snapshots for search branches (ADR-008).
//!
//! See `decisions/ADR-008-cow-ast-snapshots.md`.

use std::sync::Arc;

use crate::nodes::Composition;

/// Immutable-friendly AST view with O(1) fork via `Arc`.
#[derive(Clone, Debug)]
pub struct AstSnapshot {
    inner: Arc<Composition>,
}

impl AstSnapshot {
    #[must_use]
    pub fn new(composition: Composition) -> Self {
        Self {
            inner: Arc::new(composition),
        }
    }

    /// Shallow fork sharing unmodified subtrees.
    #[must_use]
    pub fn fork(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }

    #[must_use]
    pub fn composition(&self) -> &Composition {
        &self.inner
    }

    /// Mutable access with copy-on-write when this snapshot is shared.
    pub fn composition_mut(&mut self) -> &mut Composition {
        Arc::make_mut(&mut self.inner)
    }

    #[must_use]
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl From<Composition> for AstSnapshot {
    fn from(composition: Composition) -> Self {
        Self::new(composition)
    }
}

impl From<&Composition> for AstSnapshot {
    fn from(composition: &Composition) -> Self {
        Self::new(composition.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::CompositionBuilder;

    #[test]
    fn fork_shares_arc_until_mutation() {
        let comp = CompositionBuilder::new().one_measure().build();
        let snap = AstSnapshot::new(comp);
        let mut branch = snap.fork();
        assert_eq!(snap.strong_count(), 2);

        branch
            .composition_mut()
            .metadata
            .title = "Branch".into();
        assert_eq!(snap.composition().metadata.title, "Untitled");
        assert_eq!(branch.composition().metadata.title, "Branch");
    }

    #[test]
    fn snapshot_from_composition_reference() {
        let comp = CompositionBuilder::new().one_measure().build();
        let snap = AstSnapshot::from(&comp);
        assert_eq!(snap.composition().schema_version.major, 0);
    }
}
