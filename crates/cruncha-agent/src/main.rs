// SPDX-License-Identifier: GPL-3.0-or-later
//! `cruncha-agent`: the built-in agent as its own program (the app also runs it as
//! `cruncha-app --acp-agent …`).

fn main() -> agent_client_protocol::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // `--mcp-stdio <url>`: the stdio bridge to Cruncha's MCP server.
    if args.first().map(String::as_str) == Some("--mcp-stdio") {
        let url = args.get(1).expect("--mcp-stdio <url>");
        cruncha_agent::bridge::run(url).map_err(|e| agent_client_protocol::Error::internal_error().data(e.to_string()))?;
        return Ok(());
    }
    cruncha_agent::run(args)
}
