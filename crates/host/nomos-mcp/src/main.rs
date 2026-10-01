//! The `nomos-mcp` binary: serves a tool session over this process's own stdin
//! and stdout until stdin closes.
//!
//! This is the increment `nomos-api-transport`'s own doc named as still missing
//! one layer up -- "a host verb that runs a server is the next increment rather
//! than something claimed here" -- except a stdio tool transport has no listener
//! for a caller to bind first: a client launches this process itself and speaks
//! to it over the pipes it already owns, so there is nothing here for a
//! `nomos mcp serve` verb to wrap that this binary does not already do.
//!
//! The serving loop is the engine's. What this binary supplies is the catalogue
//! and the two pipes.

fn main() -> std::io::Result<()>
{
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    return xvpe_remote_call_backend_json::Serve_Tools(
        &nomos_mcp::NomosToolCatalog,
        stdin.lock(),
        stdout.lock(),
    );
}
