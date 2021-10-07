use always_assert::always;
use crossbeam_channel::select;
use crossbeam_channel::Receiver;
use std::fmt;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::dispatch::{NotificationDispatcher, RequestDispatcher};
use crate::global_state::{file_id_to_url, GlobalState};
use crate::lsp_utils::{apply_document_changes, is_cancelled, notification_is, Progress};
use crate::mem_docs::DocumentData;
use crate::{from_proto, handlers, lsp_ext, Result};
use ide_db::base_db::FileId;
use ide_db::base_db::SourceDatabase;
use lsp_server::Connection;
use lsp_types::notification::Notification;
use vfs::VfsPath;

pub fn main_loop(config: Config, connection: lsp_server::Connection) -> Result<()> {
    // Windows scheduler implements priority boosts: if thread waits for an
    // event (like a condvar), and event fires, priority of the thread is
    // temporary bumped. This optimization backfires in our case: each time the
    // `main_loop` schedules a task to run on a threadpool, the worker threads
    // gets a higher priority, and (on a machine with fewer cores) displaces the
    // main loop! We work-around this by marking the main loop as a
    // higher-priority thread.
    //
    // https://docs.microsoft.com/en-us/windows/win32/procthread/scheduling-priorities
    // https://docs.microsoft.com/en-us/windows/win32/procthread/priority-boosts
    // https://github.com/rust-analyzer/rust-analyzer/issues/2835
    #[cfg(windows)]
    unsafe {
        use winapi::um::processthreadsapi::*;
        let thread = GetCurrentThread();
        let thread_priority_above_normal = 1;
        SetThreadPriority(thread, thread_priority_above_normal);
    }

    GlobalState::new(connection.sender, config).run(connection.receiver)
}

enum Event {
    Lsp(lsp_server::Message),
    Task(Task),
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let debug_verbose_not = |not: &lsp_server::Notification, f: &mut fmt::Formatter| {
            f.debug_struct("Notification").field("method", &not.method).finish()
        };

        match self {
            Event::Lsp(lsp_server::Message::Notification(not)) => {
                if notification_is::<lsp_types::notification::DidOpenTextDocument>(not)
                    || notification_is::<lsp_types::notification::DidChangeTextDocument>(not)
                {
                    return debug_verbose_not(not, f);
                }
            }
            Event::Task(Task::Response(resp)) => {
                return f
                    .debug_struct("Response")
                    .field("id", &resp.id)
                    .field("error", &resp.error)
                    .finish();
            }
            _ => (),
        }
        match self {
            Event::Lsp(it) => fmt::Debug::fmt(it, f),
            Event::Task(it) => fmt::Debug::fmt(it, f),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Task {
    Response(lsp_server::Response),
    Diagnostics(Vec<(FileId, Vec<lsp_types::Diagnostic>)>),
}

impl GlobalState {
    fn run(mut self, inbox: Receiver<lsp_server::Message>) -> Result<()> {
        if self.config.did_save_text_document_dynamic_registration() {
            let save_registration_options = lsp_types::TextDocumentSaveRegistrationOptions {
                include_text: Some(false),
                text_document_registration_options: lsp_types::TextDocumentRegistrationOptions {
                    document_selector: Some(vec![
                        lsp_types::DocumentFilter {
                            language: None,
                            scheme: None,
                            pattern: Some("**/*.move".into()),
                        },
                        lsp_types::DocumentFilter {
                            language: None,
                            scheme: None,
                            pattern: Some("**/Dove.toml".into()),
                        },
                    ]),
                },
            };
            let registration = lsp_types::Registration {
                id: "textDocument/didSave".to_string(),
                method: "textDocument/didSave".to_string(),
                register_options: Some(serde_json::to_value(save_registration_options).unwrap()),
            };
            self.send_request::<lsp_types::request::RegisterCapability>(
                lsp_types::RegistrationParams { registrations: vec![registration] },
                |_, _| (),
            );
        }

        self.fetch_workspaces();

        while let Some(event) = self.next_event(&inbox) {
            if let Event::Lsp(lsp_server::Message::Notification(not)) = &event {
                if not.method == lsp_types::notification::Exit::METHOD {
                    return Ok(());
                }
            }
            self.handle_event(event)?
        }
        Err("client exited without proper shutdown sequence")?
    }

    fn next_event(&self, inbox: &Receiver<lsp_server::Message>) -> Option<Event> {
        select! {
            recv(inbox) -> msg =>
                msg.ok().map(Event::Lsp),

            recv(self.task_pool.receiver) -> task =>
                Some(Event::Task(task.unwrap())),
        }
    }

    fn handle_event(&mut self, event: Event) -> Result<()> {
        let loop_start = Instant::now();

        let was_quiescent = self.is_quiescent();
        match event {
            Event::Lsp(msg) => match msg {
                lsp_server::Message::Request(req) => self.on_request(loop_start, req)?,
                lsp_server::Message::Notification(not) => {
                    self.on_notification(not)?;
                }
                lsp_server::Message::Response(resp) => self.complete_request(resp),
            },
            Event::Task(mut task) => {
                loop {
                    match task {
                        Task::Response(response) => self.respond(response),
                        Task::Diagnostics(diagnostics_per_file) => {
                            for (file_id, diagnostics) in diagnostics_per_file {
                                self.diagnostics.set_native_diagnostics(file_id, diagnostics)
                            }
                        }
                    }
                    // Coalesce multiple task events into one loop turn
                    task = match self.task_pool.receiver.try_recv() {
                        Ok(task) => task,
                        Err(_) => break,
                    };
                }
            }
        }

        let state_changed = self.process_changes();
        let memdocs_added_or_removed = self.mem_docs.take_changes();

        if self.is_quiescent() {
            if !was_quiescent || state_changed || memdocs_added_or_removed {
                self.update_diagnostics();
            }
        }

        if let Some(diagnostic_changes) = self.diagnostics.take_changes() {
            for file_id in diagnostic_changes {
                let db = self.analysis_host.raw_database();
                let source_root = db.file_source_root(file_id);
                if db.source_root(source_root).is_library {
                    // Only publish diagnostics for files in the workspace, not from crates.io deps
                    // or the sysroot.
                    // While theoretically these should never have errors, we have quite a few false
                    // positives particularly in the stdlib, and those diagnostics would stay around
                    // forever if we emitted them here.
                    continue;
                }

                let url = file_id_to_url(&self.vfs.read().0, file_id);
                let diagnostics = self.diagnostics.diagnostics_for(file_id).cloned().collect();
                let version = from_proto::vfs_path(&url)
                    .map(|path| self.mem_docs.get(&path).map(|it| it.version))
                    .unwrap_or_default();

                self.send_notification::<lsp_types::notification::PublishDiagnostics>(
                    lsp_types::PublishDiagnosticsParams { uri: url, diagnostics, version },
                );
            }
        }

        let status = self.current_status();
        if self.last_reported_status.as_ref() != Some(&status) {
            self.last_reported_status = Some(status.clone());

            if let (lsp_ext::Health::Error, Some(message)) = (status.health, &status.message) {
                self.show_message(lsp_types::MessageType::Error, message.clone());
            }
        }

        let loop_duration = loop_start.elapsed();
        if loop_duration > Duration::from_millis(100) {
            tracing::warn!("overly long loop turn: {:?}", loop_duration);
        }
        Ok(())
    }

    fn on_request(&mut self, request_received: Instant, req: lsp_server::Request) -> Result<()> {
        self.register_request(&req, request_received);

        if self.shutdown_requested {
            self.respond(lsp_server::Response::new_err(
                req.id,
                lsp_server::ErrorCode::InvalidRequest as i32,
                "Shutdown already requested.".to_owned(),
            ));

            return Ok(());
        }

        RequestDispatcher { req: Some(req), global_state: self }
            .on_sync_mut::<lsp_types::request::Shutdown>(|s, ()| {
                s.shutdown_requested = true;
                Ok(())
            })?
            .finish();
        Ok(())
    }

    fn on_notification(&mut self, not: lsp_server::Notification) -> Result<()> {
        NotificationDispatcher { not: Some(not), global_state: self }
            .on::<lsp_types::notification::Cancel>(|this, params| {
                let id: lsp_server::RequestId = match params.id {
                    lsp_types::NumberOrString::Number(id) => id.into(),
                    lsp_types::NumberOrString::String(id) => id.into(),
                };
                this.cancel(id);
                Ok(())
            })?
            .on::<lsp_types::notification::WorkDoneProgressCancel>(|_this, _params| {
                // Just ignore this. It is OK to continue sending progress
                // notifications for this token, as the client can't know when
                // we accepted notification.
                Ok(())
            })?
            .on::<lsp_types::notification::DidOpenTextDocument>(|this, params| {
                if let Ok(path) = from_proto::vfs_path(&params.text_document.uri) {
                    if this
                        .mem_docs
                        .insert(path.clone(), DocumentData::new(params.text_document.version))
                        .is_err()
                    {
                        tracing::error!("duplicate DidOpenTextDocument: {}", path)
                    }
                    this.vfs
                        .write()
                        .0
                        .set_file_contents(path, Some(params.text_document.text.into_bytes()));
                }
                Ok(())
            })?
            .on::<lsp_types::notification::DidChangeTextDocument>(|this, params| {
                if let Ok(path) = from_proto::vfs_path(&params.text_document.uri) {
                    match this.mem_docs.get_mut(&path) {
                        Some(doc) => {
                            // The version passed in DidChangeTextDocument is the version after all edits are applied
                            // so we should apply it before the vfs is notified.
                            doc.version = params.text_document.version;
                        }
                        None => {
                            tracing::error!("unexpected DidChangeTextDocument: {}; send DidOpenTextDocument first", path);
                            return Ok(());
                        }
                    };

                    let vfs = &mut this.vfs.write().0;
                    let file_id = vfs.file_id(&path).unwrap();
                    let mut text = String::from_utf8(vfs.file_contents(file_id).to_vec()).unwrap();
                    apply_document_changes(&mut text, params.content_changes);

                    vfs.set_file_contents(path, Some(text.into_bytes()));
                }
                Ok(())
            })?
            .on::<lsp_types::notification::DidCloseTextDocument>(|this, params| {
                if let Ok(path) = from_proto::vfs_path(&params.text_document.uri) {
                    if this.mem_docs.remove(&path).is_err() {
                        tracing::error!("orphan DidCloseTextDocument: {}", path);
                    }
                }
                Ok(())
            })?
            .finish();
        Ok(())
    }

    fn update_diagnostics(&mut self) {
        let subscriptions = self
            .mem_docs
            .iter()
            .map(|path| self.vfs.read().0.file_id(path).unwrap())
            .collect::<Vec<_>>();

        tracing::trace!("updating notifications for {:?}", subscriptions);

        let snapshot = self.snapshot();
        self.task_pool.handle.spawn(move || {
            let diagnostics = subscriptions
                .into_iter()
                .filter_map(|file_id| {
                    handlers::publish_diagnostics(&snapshot, file_id)
                        .map_err(|err| {
                            if !is_cancelled(&*err) {
                                tracing::error!("failed to compute diagnostics: {:?}", err);
                            }
                            ()
                        })
                        .ok()
                        .map(|diags| (file_id, diags))
                })
                .collect::<Vec<_>>();
            Task::Diagnostics(diagnostics)
        })
    }
}
