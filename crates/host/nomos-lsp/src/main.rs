//! The `nomos-lsp` binary: serves a language-server session over this process's own stdin
//! and stdout until the client shuts it down -- the same launch shape `nomos-mcp`'s own
//! binary has, a client launching this process and speaking to it over the pipes it already
//! owns.
//!
//! The server loop is the engine's. What this binary supplies is the thing that judges.

fn main() -> std::io::Result<()>
{
    let mut provider = nomos_lsp::NomosDiagnosticProvider::New();

    return xvpe_language_server_backend_lsp::Run_Server(&mut provider);
}
