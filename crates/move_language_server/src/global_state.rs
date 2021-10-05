//! The context or environment in which the language server functions. In our
//! server implementation this is know as the `WorldState`.
//!
//! Each tick provides an immutable snapshot of the state as `WorldSnapshot`.

use std::{sync::Arc, time::Instant};

use crate::diagnostics::DiagnosticCollection;
use crate::line_index::{LineEndings, LineIndex, OffsetEncoding};
use crate::main_loop::Task;
use crate::mem_docs::MemDocs;
use crate::reload::SourceRootConfig;
use crate::thread_pool::TaskPool;
use crate::to_proto::url_from_abs_path;
use crate::Result;
use crate::{from_proto, lsp_ext, reload};
use crossbeam_channel::{unbounded, Receiver, Sender};
use ide::{Analysis, AnalysisHost, Cancellable};
use ide_db::base_db::{Change, FileId};
use lsp_types::{SemanticTokens, Url};
use parking_lot::{Mutex, RwLock};
use rustc_hash::FxHashMap;
use vfs::AnchoredPathBuf;
use crate::config::Config;

// Enforces drop order
pub(crate) struct Handle<H, C> {
    pub(crate) handle: H,
    pub(crate) receiver: C,
}

pub(crate) type ReqHandler = fn(&mut GlobalState, lsp_server::Response);
pub(crate) type ReqQueue = lsp_server::ReqQueue<(String, Instant), ReqHandler>;

/// `GlobalState` is the primary mutable state of the language server
///
/// The most interesting components are `vfs`, which stores a consistent
/// snapshot of the file systems, and `analysis_host`, which stores our
/// incremental salsa database.
///
/// Note that this struct has more than on impl in various modules!
pub(crate) struct GlobalState {
    sender: Sender<lsp_server::Message>,
    req_queue: ReqQueue,
    pub(crate) task_pool: Handle<TaskPool<Task>, Receiver<Task>>,
    pub(crate) config: Arc<Config>,
    pub(crate) analysis_host: AnalysisHost,
    pub(crate) diagnostics: DiagnosticCollection,
    pub(crate) mem_docs: MemDocs,
    pub(crate) shutdown_requested: bool,
    pub(crate) last_reported_status: Option<lsp_ext::ServerStatusParams>,
    pub(crate) source_root_config: SourceRootConfig,

    pub(crate) vfs: Arc<RwLock<(vfs::Vfs, FxHashMap<FileId, LineEndings>)>>,
    pub(crate) vfs_config_version: u32,
    pub(crate) vfs_progress_config_version: u32,
    pub(crate) vfs_progress_n_total: usize,
    pub(crate) vfs_progress_n_done: usize,
}

/// An immutable snapshot of the world's state at a point in time.
pub(crate) struct GlobalStateSnapshot {
    pub(crate) analysis: Analysis,
    mem_docs: MemDocs,
    vfs: Arc<RwLock<(vfs::Vfs, FxHashMap<FileId, LineEndings>)>>,
}

impl std::panic::UnwindSafe for GlobalStateSnapshot {}

impl GlobalState {
    pub(crate) fn new(sender: Sender<lsp_server::Message>, config: Config) -> GlobalState {
        let task_pool = {
            let (sender, receiver) = unbounded();
            let handle = TaskPool::new(sender);
            Handle { handle, receiver }
        };

        let analysis_host = AnalysisHost::new();
        let mut this = GlobalState {
            sender,
            req_queue: ReqQueue::default(),
            task_pool,
            config,
            analysis_host,
            diagnostics: Default::default(),
            mem_docs: MemDocs::default(),
            shutdown_requested: false,
            last_reported_status: None,
            source_root_config: SourceRootConfig::default(),

            vfs: Arc::new(RwLock::new((vfs::Vfs::default(), FxHashMap::default()))),
            vfs_config_version: 0,
            vfs_progress_config_version: 0,
            vfs_progress_n_total: 0,
            vfs_progress_n_done: 0,
        };
        this
    }

    pub(crate) fn process_changes(&mut self) -> bool {
        let mut fs_changes = Vec::new();
        // A file was added or deleted
        let mut has_structure_changes = false;

        let change = {
            let mut change = Change::new();
            let (vfs, line_endings_map) = &mut *self.vfs.write();
            let changed_files = vfs.take_changes();
            if changed_files.is_empty() {
                return false;
            }

            for file in changed_files {
                if let Some(path) = vfs.file_path(file.file_id).as_path() {
                    let path = path.to_path_buf();
                    fs_changes.push((path, file.change_kind));
                    if file.is_created_or_deleted() {
                        has_structure_changes = true;
                    }
                }

                let text = if file.exists() {
                    let bytes = vfs.file_contents(file.file_id).to_vec();
                    match String::from_utf8(bytes).ok() {
                        Some(text) => {
                            let (text, line_endings) = LineEndings::normalize(text);
                            line_endings_map.insert(file.file_id, line_endings);
                            Some(Arc::new(text))
                        }
                        None => None,
                    }
                } else {
                    None
                };
                change.change_file(file.file_id, text);
            }
            if has_structure_changes {
                let roots = self.source_root_config.partition(vfs);
                change.set_roots(roots);
            }
            change
        };

        self.analysis_host.apply_change(change);
        true
    }

    pub(crate) fn snapshot(&self) -> GlobalStateSnapshot {
        GlobalStateSnapshot {
            analysis: self.analysis_host.analysis(),
            vfs: Arc::clone(&self.vfs),
            mem_docs: self.mem_docs.clone(),
        }
    }

    pub(crate) fn send_request<R: lsp_types::request::Request>(
        &mut self,
        params: R::Params,
        handler: ReqHandler,
    ) {
        let request = self.req_queue.outgoing.register(R::METHOD.to_string(), params, handler);
        self.send(request.into());
    }
    pub(crate) fn complete_request(&mut self, response: lsp_server::Response) {
        let handler = self.req_queue.outgoing.complete(response.id.clone());
        handler(self, response)
    }

    pub(crate) fn send_notification<N: lsp_types::notification::Notification>(
        &mut self,
        params: N::Params,
    ) {
        let not = lsp_server::Notification::new(N::METHOD.to_string(), params);
        self.send(not.into());
    }

    pub(crate) fn register_request(
        &mut self,
        request: &lsp_server::Request,
        request_received: Instant,
    ) {
        self.req_queue
            .incoming
            .register(request.id.clone(), (request.method.clone(), request_received));
    }
    pub(crate) fn respond(&mut self, response: lsp_server::Response) {
        if self.req_queue.incoming.complete(response.id.clone()).is_some() {
            self.send(response.into());
        }
    }
    pub(crate) fn cancel(&mut self, request_id: lsp_server::RequestId) {
        if let Some(response) = self.req_queue.incoming.cancel(request_id) {
            self.send(response.into());
        }
    }

    fn send(&mut self, message: lsp_server::Message) {
        self.sender.send(message).unwrap()
    }
}

impl Drop for GlobalState {
    fn drop(&mut self) {
        self.analysis_host.request_cancellation()
    }
}

impl GlobalStateSnapshot {
    pub(crate) fn url_to_file_id(&self, url: &Url) -> Result<FileId> {
        url_to_file_id(&self.vfs.read().0, url)
    }

    pub(crate) fn file_id_to_url(&self, id: FileId) -> Url {
        file_id_to_url(&self.vfs.read().0, id)
    }

    pub(crate) fn file_line_index(&self, file_id: FileId) -> Cancellable<LineIndex> {
        let endings = self.vfs.read().1[&file_id];
        let index = self.analysis.file_line_index(file_id)?;
        let res = LineIndex { index, endings, encoding: OffsetEncoding::Utf8 };
        Ok(res)
    }

    pub(crate) fn url_file_version(&self, url: &Url) -> Option<i32> {
        let path = from_proto::vfs_path(url).ok()?;
        Some(self.mem_docs.get(&path)?.version)
    }

    pub(crate) fn anchored_path(&self, path: &AnchoredPathBuf) -> Url {
        let mut base = self.vfs.read().0.file_path(path.anchor);
        base.pop();
        let path = base.join(&path.path).unwrap();
        let path = path.as_path().unwrap();
        url_from_abs_path(path)
    }
}

pub(crate) fn file_id_to_url(vfs: &vfs::Vfs, id: FileId) -> Url {
    let path = vfs.file_path(id);
    let path = path.as_path().unwrap();
    url_from_abs_path(path)
}

pub(crate) fn url_to_file_id(vfs: &vfs::Vfs, url: &Url) -> Result<FileId> {
    let path = from_proto::vfs_path(url)?;
    let res = vfs.file_id(&path).ok_or_else(|| format!("file not found: {}", path))?;
    Ok(res)
}
