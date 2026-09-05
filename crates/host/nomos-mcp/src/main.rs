//! The `nomos-mcp` binary: serves MCP requests over this process's own stdin and stdout
//! until stdin closes.
//!
//! This is the increment `nomos-api-transport`'s own doc named as still missing one layer
//! up -- "a host verb that runs a server is the next increment rather than something
//! claimed here" -- except MCP's stdio transport has no listener for a caller to bind
//! first: an MCP client launches this process itself and speaks to it over the pipes it
//! already owns, so there is nothing here for a `nomos mcp serve` verb to wrap that this
//! binary does not already do on its own.

fn main() -> std::io::Result<()>
{
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    return nomos_mcp::Serve(stdin.lock(), stdout.lock());
}
