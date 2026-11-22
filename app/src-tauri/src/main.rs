// SPDX-License-Identifier: GPL-3.0-or-later
// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // The built-in agent (ADR-0009): the agent panel starts this program as an ACP agent.
    let mut args = std::env::args().skip(1);
    let first = args.next();
    // The stdio bridge to Cruncha's MCP server, for agents without HTTP MCP.
    if first.as_deref() == Some("--mcp-stdio") {
        let url = args.next().unwrap_or_default();
        if let Err(e) = cruncha_agent::bridge::run(&url) {
            eprintln!("cruncha: MCP bridge: {e}");
            std::process::exit(1);
        }
        return;
    }
    if first.as_deref() == Some("--acp-agent") {
        if let Err(e) = cruncha_agent::run(args.collect()) {
            eprintln!("cruncha-agent: {e}");
            std::process::exit(1);
        }
        return;
    }
    cruncha_app_lib::run()
}
