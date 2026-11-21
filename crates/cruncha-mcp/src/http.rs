// SPDX-License-Identifier: GPL-3.0-or-later

//! Serving the tools over MCP Streamable HTTP, for agents on this computer.
//!
//! - Bound to 127.0.0.1 on a free port: nothing off this machine can connect.
//! - Every request needs `Authorization: Bearer <token>` (a fresh random token per start).
//! - The `Host` header must name the loopback address, and requests that carry a browser
//!   `Origin` are refused, so a web page can't reach the server (DNS rebinding).

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use tokio_util::sync::CancellationToken;

use crate::Backend;
use crate::tools::CrunchaTools;

/// A running server. Dropping it doesn't stop it; call [`Server::stop`].
pub struct Server {
    /// `http://127.0.0.1:PORT/mcp`
    pub url: String,
    pub token: String,
    pub port: u16,
    stop: CancellationToken,
}

impl Server {
    pub fn stop(&self) {
        self.stop.cancel();
    }

    /// The command that registers this server with Claude Code.
    pub fn claude_command(&self) -> String {
        format!(
            "claude mcp add --transport http cruncha {} --header \"Authorization: Bearer {}\"",
            self.url, self.token
        )
    }

    /// A `mcpServers` entry for agents configured with JSON (Claude Desktop-style, Gemini CLI,
    /// ACP `session/new`).
    pub fn config_json(&self) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "mcpServers": {
                "cruncha": {
                    "type": "http",
                    "url": self.url,
                    "headers": { "Authorization": format!("Bearer {}", self.token) }
                }
            }
        }))
        .unwrap_or_default()
    }
}

struct Guard {
    token: String,
    port: u16,
}

async fn guard(State(g): State<Arc<Guard>>, req: Request, next: Next) -> Response {
    let headers = req.headers();
    let host_ok = headers
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|h| h == format!("127.0.0.1:{}", g.port) || h == format!("localhost:{}", g.port));
    if !host_ok {
        return (StatusCode::FORBIDDEN, "wrong host").into_response();
    }
    if headers.contains_key(header::ORIGIN) {
        return (StatusCode::FORBIDDEN, "browser requests are not accepted").into_response();
    }
    let auth_ok = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .is_some_and(|t| constant_time_eq(t.as_bytes(), g.token.as_bytes()));
    if !auth_ok {
        return (StatusCode::UNAUTHORIZED, "missing or wrong token").into_response();
    }
    next.run(req).await
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Start serving `backend`'s document on a free loopback port.
pub async fn serve<B: Backend>(backend: B) -> std::io::Result<Server> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let token = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
    let stop = CancellationToken::new();
    let config = StreamableHttpServerConfig::default().with_cancellation_token(stop.child_token());
    let service: StreamableHttpService<CrunchaTools<B>, LocalSessionManager> =
        StreamableHttpService::new(move || Ok(CrunchaTools::new(backend.clone())), Default::default(), config);
    let g = Arc::new(Guard { token: token.clone(), port });
    let router = axum::Router::new()
        .nest_service("/mcp", service)
        .layer(middleware::from_fn_with_state(g, guard));
    let s = stop.clone();
    tokio::spawn(async move {
        let _ = axum::serve(listener, router)
            .with_graceful_shutdown(async move { s.cancelled_owned().await })
            .await;
    });
    Ok(Server { url: format!("http://127.0.0.1:{port}/mcp"), token, port, stop })
}
