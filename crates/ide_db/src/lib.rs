mod apply_change;
mod line_index;

pub use line_index::{LineCol, LineColUtf16, LineIndex};

use base_db::{salsa, FileId};
use std::sync::Arc;
use std::{fmt, mem::ManuallyDrop};

pub use base_db;
use base_db::salsa::Durability;
use base_db::SourceDatabase;

#[salsa::database(base_db::SourceDatabaseStorage, LineIndexDatabaseStorage)]
pub struct RootDatabase {
    // We use `ManuallyDrop` here because every codegen unit that contains a
    // `&RootDatabase -> &dyn OtherDatabase` cast will instantiate its drop glue in the vtable,
    // which duplicates `Weak::drop` and `Arc::drop` tens of thousands of times, which makes
    // compile times of all `ide_*` and downstream crates suffer greatly.
    storage: ManuallyDrop<salsa::Storage<RootDatabase>>,
}

impl Drop for RootDatabase {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.storage);
        }
    }
}

impl fmt::Debug for RootDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RootDatabase").finish()
    }
}

impl salsa::Database for RootDatabase {}

impl Default for RootDatabase {
    fn default() -> RootDatabase {
        RootDatabase::new(None)
    }
}

impl RootDatabase {
    pub fn new(lru_capacity: Option<usize>) -> RootDatabase {
        let mut db = RootDatabase { storage: ManuallyDrop::new(salsa::Storage::default()) };
        db.set_local_roots_with_durability(Default::default(), Durability::HIGH);
        db.set_library_roots_with_durability(Default::default(), Durability::HIGH);
        db.update_lru_capacity(lru_capacity);
        db
    }

    pub fn update_lru_capacity(&mut self, lru_capacity: Option<usize>) {
        let lru_capacity = lru_capacity.unwrap_or(base_db::DEFAULT_LRU_CAP);
        base_db::ParseQuery.in_db_mut(self).set_lru_capacity(lru_capacity);
    }
}

impl salsa::ParallelDatabase for RootDatabase {
    fn snapshot(&self) -> salsa::Snapshot<RootDatabase> {
        salsa::Snapshot::new(RootDatabase { storage: ManuallyDrop::new(self.storage.snapshot()) })
    }
}

#[salsa::query_group(LineIndexDatabaseStorage)]
pub trait LineIndexDatabase: base_db::SourceDatabase {
    fn line_index(&self, file_id: FileId) -> Arc<LineIndex>;
}

fn line_index(db: &dyn LineIndexDatabase, file_id: FileId) -> Arc<LineIndex> {
    let text = db.file_text(file_id);
    Arc::new(LineIndex::new(&*text))
}
