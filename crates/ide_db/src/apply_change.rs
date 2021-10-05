use crate::RootDatabase;
use base_db::salsa::{Database, Durability};
use base_db::{Change, SourceDatabase, SourceRootId};
use rustc_hash::FxHashSet;
use std::sync::Arc;

impl RootDatabase {
    pub fn request_cancellation(&mut self) {
        self.salsa_runtime_mut().synthetic_write(Durability::LOW);
    }

    pub fn apply_change(&mut self, change: Change) {
        self.request_cancellation();
        if let Some(roots) = &change.roots {
            let mut local_roots = FxHashSet::default();
            let mut library_roots = FxHashSet::default();
            for (idx, root) in roots.iter().enumerate() {
                let root_id = SourceRootId(idx as u32);
                if root.is_library {
                    library_roots.insert(root_id);
                } else {
                    local_roots.insert(root_id);
                }
            }
            self.set_local_roots_with_durability(Arc::new(local_roots), Durability::HIGH);
            self.set_library_roots_with_durability(Arc::new(library_roots), Durability::HIGH);
        }
        change.apply(self);
    }
}
