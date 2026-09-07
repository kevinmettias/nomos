//! The server loop: `initialize`, then a full re-check and `textDocument/publishDiagnostics`
//! on every `didOpen` or `didSave`, until the client asks this process to shut down.
//!
//! This is the whole of what makes this crate a language server rather than a library of
//! pure functions. It owns no judgment: every diagnostic it publishes is
//! [`crate::file_diagnostic::Diagnostics_For`]'s own translation of a `Finding`
//! `nomos_check_orchestration::Run` already produced. What is real here and untested by
//! this crate's own unit tests -- the JSON-RPC handshake, the file walk, the subprocess
//! launches `Run` makes on this repository's behalf -- is exactly the composition-root
//! plumbing `nomos-cli::check` and `nomos-correction-orchestration::run` already carry for
//! themselves, not a new design.

use crate::build_variant::Host_Variant;
use crate::file_diagnostic::Diagnostics_For;
use crate::sources::Walked_Sources;
use crate::uri::{File_Uri_From_Path, Path_From_File_Uri};
use lsp_server::{Connection, Message};
use lsp_types::notification::{DidOpenTextDocument, DidSaveTextDocument, Notification, PublishDiagnostics};
use lsp_types::{Diagnostic, InitializeParams, PublishDiagnosticsParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind};
use nomos_analysis::MemoryFactStore;
use nomos_platform_std::{StdFileSystem, StdProcessLauncher};
use nomos_workspace::Workspace;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

/// Runs this server over its own process's stdin and stdout until the client shuts it down.
///
/// # Errors
///
/// Returns the first transport failure -- a malformed frame, or a pipe closed mid-message.
/// An ordinary client disconnect after `exit` is not one; `main.rs` is the only caller and
/// a JSON-RPC failure there is worth a nonzero exit, the same as any other host binary in
/// this workspace reporting a real failure through its own exit code rather than a panic.
pub fn Run_Server() -> std::io::Result<()>
{
    let (connection, io_threads) = Connection::stdio();

    let root = match Initialize(&connection)
    {
        Ok(root) => root,
        Err(protocol_error) => return Err(std::io::Error::other(protocol_error.to_string())),
    };

    let mut state = ServerState::New();
    Serve(&connection, &root, &mut state);

    io_threads.join()?;
    return Ok(());
}

/// Performs the `initialize` / `initialized` handshake and answers with this server's own
/// capabilities: full-document sync, since every re-check here re-walks the whole tree
/// rather than reading an incremental edit.
fn Initialize(connection: &Connection) -> Result<PathBuf, Box<dyn std::error::Error + Sync + Send>>
{
    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..ServerCapabilities::default()
    };
    let server_capabilities = serde_json::to_value(capabilities)?;
    let initialize_params = connection.initialize(server_capabilities)?;
    let initialize_params: InitializeParams = serde_json::from_value(initialize_params)?;

    return Ok(Workspace_Root(&initialize_params));
}

/// The root this server checks: the first declared workspace folder, falling back to the
/// deprecated single `rootUri`, falling back to this process's own current directory when a
/// client (unusually) names neither.
fn Workspace_Root(params: &InitializeParams) -> PathBuf
{
    if let Some(folder) = params.workspace_folders.as_ref().and_then(|folders| return folders.first())
        && let Some(path) = Path_From_File_Uri(&folder.uri)
    {
        return path;
    }

    // `root_uri` is deprecated by the LSP spec in favor of `workspace_folders`, but an
    // older or simpler client may still send only this field, and falling back to it is
    // this function's own stated fallback order.
    #[allow(deprecated)]
    if let Some(root_uri) = params.root_uri.as_ref()
        && let Some(path) = Path_From_File_Uri(root_uri)
    {
        return path;
    }

    return std::env::current_dir().unwrap_or_else(|_| return PathBuf::from("."));
}

/// Everything [`Serve`]'s loop carries across calls to [`Recheck_And_Publish`], for the life
/// of one server process -- grouped into one value so [`Serve`] and [`Recheck_And_Publish`]
/// each stay within this crate's own parameter-count limit.
///
/// `workspace` and `store` are reused across every recheck rather than rebuilt per call --
/// `P14-ANALYSIS-009-STORE-WORKSPACE-REUSE-FIRST-INCREMENT` gave `nomos_check_orchestration::
/// Run` a caller-supplied `workspace`/`store` for exactly this, and this server is this
/// workspace's first real caller with a process lifetime long enough to hold either across
/// two calls: `OD-ANALYSIS-009`'s own second amendment names this crate as the concrete case
/// its first trigger was written for. Reused, not persisted -- both still end when this
/// process does, the same as every other caller in this workspace; nothing here asks either
/// to survive past that.
struct ServerState
{
    /// The set of files this server has ever sent a diagnostic for, so a file that goes
    /// clean gets its diagnostics cleared rather than left stale forever.
    published: HashSet<String>,
    workspace: Option<Workspace>,
    store: MemoryFactStore,
}

impl ServerState
{
    fn New() -> Self
    {
        return Self { published: HashSet::new(), workspace: None, store: MemoryFactStore::New() };
    }
}

/// The main loop: a full re-check and republish on every `didOpen` or `didSave`, and a
/// clean exit on `shutdown`/`exit`.
fn Serve(connection: &Connection, root: &Path, state: &mut ServerState)
{
    for message in &connection.receiver
    {
        match message
        {
            Message::Request(request) =>
            {
                match connection.handle_shutdown(&request)
                {
                    Ok(true) | Err(_) => return,
                    Ok(false) => {}
                }
            }
            Message::Notification(notification) =>
            {
                if notification.method == DidOpenTextDocument::METHOD || notification.method == DidSaveTextDocument::METHOD
                {
                    Recheck_And_Publish(connection, root, state);
                }
            }
            Message::Response(_) => {}
        }
    }
}

/// Walks `root`, runs every composed rule over it, and publishes what
/// [`Diagnostics_For`] makes of the result -- one `publishDiagnostics` notification per
/// file that has a diagnostic now or had one before this call.
///
/// `state.workspace` and `state.store` carry across every call this server ever makes,
/// rather than starting fresh each time -- see [`ServerState`]'s own doc for why.
fn Recheck_And_Publish(connection: &Connection, root: &Path, state: &mut ServerState)
{
    let Some(sources) = Walked_Sources(root)
    else
    {
        return;
    };

    let outcome = nomos_check_orchestration::Run(
        &sources,
        nomos_check_orchestration::RunContext {
            variant: Host_Variant(),
            root,
            launcher: &StdProcessLauncher,
            filesystem: &StdFileSystem,
            workspace: &mut state.workspace,
            store: &mut state.store,
        },
        &[],
    );

    let nomos_check_orchestration::CheckOutcome::Judged { findings, .. } = outcome
    else
    {
        return;
    };

    let mut by_path: BTreeMap<String, Vec<Diagnostic>> = BTreeMap::new();
    for finding in &findings
    {
        for file_diagnostic in Diagnostics_For(finding)
        {
            by_path.entry(file_diagnostic.path).or_default().push(file_diagnostic.diagnostic);
        }
    }

    let mut still_published = HashSet::new();
    for (path, diagnostics) in &by_path
    {
        Publish(connection, root, path, diagnostics.clone());
        still_published.insert(path.clone());
    }
    for stale in state.published.difference(&still_published)
    {
        Publish(connection, root, stale, Vec::new());
    }

    state.published = still_published;
}

/// Sends one `textDocument/publishDiagnostics` notification for `path`, resolved against
/// `root` -- skipped silently when `path` cannot be rendered as a file URI, the one case
/// [`File_Uri_From_Path`] refuses (a path that is not valid UTF-8, which none of this
/// workspace's own repo-relative paths ever are).
fn Publish(connection: &Connection, root: &Path, path: &str, diagnostics: Vec<Diagnostic>)
{
    let Some(uri) = File_Uri_From_Path(&root.join(path))
    else
    {
        return;
    };

    let params = PublishDiagnosticsParams { uri, diagnostics, version: None };
    let Ok(params) = serde_json::to_value(params)
    else
    {
        return;
    };

    let notification = lsp_server::Notification { method: PublishDiagnostics::METHOD.to_owned(), params };
    let _ignored = connection.sender.send(Message::Notification(notification));
}

#[cfg(test)]
mod tests
{
    use super::{Recheck_And_Publish, ServerState};
    use lsp_server::Connection;

    /// The pair `Connection::memory()` returns is unbounded and this crate's own doc names
    /// it as this crate's intended test seam ("use this for testing"), so `Recheck_And_
    /// Publish`'s own `connection.sender.send` calls neither block nor need a reader on the
    /// other end for this test to observe `state` afterward.
    fn Root(name: &str) -> std::path::PathBuf
    {
        let root = std::env::temp_dir().join(name);
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("creates a fresh directory");
        return root;
    }

    #[test]
    fn Test_Recheck_And_Publish_Should_Reuse_The_Same_Workspace_And_Advance_Its_Generation_On_A_Real_Edit()
    {
        let root = Root("nomos-lsp-server-reuse-edited-root");
        std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("writable");

        let (connection, _client) = Connection::memory();
        let mut state = ServerState::New();

        Recheck_And_Publish(&connection, &root, &mut state);
        let generation_after_first = state.workspace.as_ref().expect("a walked tree must leave a workspace behind").Generation();

        std::fs::write(root.join("a.rs"), "pub fn Ok() {}\npub fn Also_Ok() {}\n").expect("writable");
        Recheck_And_Publish(&connection, &root, &mut state);
        let generation_after_second = state.workspace.as_ref().expect("a walked tree must leave a workspace behind").Generation();

        let _ignored = std::fs::remove_dir_all(&root);
        assert!(
            generation_after_second > generation_after_first,
            "a real edit reusing the same workspace must advance its generation, not repeat it: \
             {generation_after_first:?} then {generation_after_second:?}"
        );
    }

    #[test]
    fn Test_Recheck_And_Publish_Should_Not_Advance_Generation_A_Second_Time_Over_An_Untouched_Tree()
    {
        let root = Root("nomos-lsp-server-reuse-untouched-root");
        std::fs::write(root.join("a.rs"), "pub fn Ok() {}\n").expect("writable");

        let (connection, _client) = Connection::memory();
        let mut state = ServerState::New();

        Recheck_And_Publish(&connection, &root, &mut state);
        let generation_after_first = state.workspace.as_ref().expect("a walked tree must leave a workspace behind").Generation();

        Recheck_And_Publish(&connection, &root, &mut state);
        let generation_after_second = state.workspace.as_ref().expect("a walked tree must leave a workspace behind").Generation();

        let _ignored = std::fs::remove_dir_all(&root);
        assert_eq!(
            generation_after_second, generation_after_first,
            "an untouched file must not advance the generation a second time: \
             {generation_after_first:?} then {generation_after_second:?}"
        );
    }
}
