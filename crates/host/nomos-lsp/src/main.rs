//! The `nomos-lsp` binary: serves LSP requests over this process's own stdin and stdout
//! until the client shuts it down, the same launch shape `nomos-mcp`'s own binary already
//! has for MCP -- a client launches this process itself and speaks to it over the pipes it
//! already owns.

fn main() -> std::io::Result<()>
{
    return nomos_lsp::Run_Server();
}
