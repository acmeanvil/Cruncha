# ADR-0009: Agent interface: ACP client, Cruncha MCP server, and a built-in agent from looking_glass

**Status:** Accepted
**Date:** 2026-11-18
**Deciders:** acmeanvil

## Context

Cruncha is AI-forward: agents read, author, check and explain documents. The engine computes every number, and agent edits are proposals (design 02 §6, `skills/cruncha-core`). Users will want **their own agent**:
- Claude Code, Codex, Gemini CLI or Qwen Code;
- or a built-in one using any model provider, including local ones.

The interface should feel modern, unlike a terminal transcript. **looking_glass** (`~/Developer/looking_glass`, same owner, MIT, about 18.6k lines of Rust) already solves most of this:
- **`lg-acp`** is a UI-agnostic **Agent Client Protocol** client. It keeps ACP schema types out of the UI behind its own neutral `Event`/`Command` model, and serves fs, terminal and permission requests with sandboxed writes. It also handles remote agents over ssh.
- **`lg-agent`** is an ACP agent with 11 provider profiles:
  - native Anthropic Messages, with prompt caching and thinking round-trips;
  - OpenAI-compatible SSE, model listing, and usage reporting;
  - an end-to-end test suite against mock providers.
- **The app's UX:** streaming Markdown, tool cards, a pending diff before each permission prompt, Allow / Always / Reject, a plan panel, model and mode menus, history and resume, and handover when switching agents.

Gaps found:
- **No MCP:** `session/new` sends no `mcpServers`, and lg-agent isn't an MCP client.
- **No skills support, and a hard-coded system prompt.**
- **Four hard-coded coding tools** (read, list, write, run).
- **Cancellation is global, not per session.**
- **No session persistence in lg-agent.**
- **lg-render has no math support.**

The licenses (MIT plus Apache-2.0, all transitive dependencies permissive) are GPL-3 compatible.

## Decision

1. **`cruncha-mcp`** is the *only* way agents touch a document. It is an MCP server inside the engine process and exposes the tools in `skills/cruncha-core`: `read_doc`, `run`, `propose_edit`, `explain`, `check`, `history` and `query_data`. Edits become proposals in the `proposals/` table, applied only by a human's commit (ADR-0016), and agents never write the container directly.
2. **Port `lg-acp`** into Cruncha's Rust backend (as `cruncha-acp`, keeping the MIT notice), and extend it:
   - pass `cruncha-mcp` in `mcpServers` on `session/new` and `session/load`;
   - **don't advertise `fs.write_text_file` or `terminal`** to agents working on a document;
   - map the ACP updates it currently drops (images, resources, available commands, mode).
3. **Built-in agent = `lg-agent`'s provider core**, run as an ACP subprocess, so Cruncha treats it exactly like Claude Code. Changes:
   - an MCP client, or a tool registry, in place of the four coding tools;
   - a system prompt per document plus a **skills loader**: `SKILL.md` front matter goes into the prompt, and the body loads on demand through `load_skill`;
   - per-session cancellation, `session/load` persistence, and context compaction.
4. **Skills** ship in `skills/` in the Agent Skills format. External agents discover them from the project, and the built-in agent loads them itself.
5. **The agent panel** is rebuilt in the Tauri web UI (ADR-0008), using looking_glass as the UX spec:
   - tool cards rendered as **cell proposals**, with a source diff and a result diff;
   - Accept / Reject per proposal;
   - a plan panel, model and mode menus, a usage pill, and a history of conversations linked to document versions.
6. **Settings** (after looking_glass's Settings): the default agent, each agent's model and connection, and provider connections whose API keys live only in the OS credential store (the `keyring` crate, MIT/Apache-2.0, as looking_glass does). A key is read when an agent starts and passed in its environment, so Claude Code, Codex and Gemini can run on an API key instead of a login, and the built-in agent gets its provider, base URL and model.
7. **Interim:** until the panel exists, use looking_glass itself as the agent console, pointing Claude Code or lg-agent at `cruncha-mcp`.

## Options Considered

### Option A: Built-in agent only (our own loop, no ACP)
**Pros:** full control.
**Cons:** users can't bring Claude Code or Codex; we would duplicate provider work that lg-agent already has.

### Option B: MCP server only (agents run wherever the user likes)
**Pros:** minimal work; works with any MCP-capable agent.
**Cons:** no in-app experience, so no proposals UI next to the document.

### Option C: ACP client plus MCP server plus a built-in ACP agent, reusing looking_glass (proposed)
**Pros:** any agent, one UI, one tool surface, reuse of tested code.
**Cons:** we maintain a fork of lg-acp and lg-agent. **Mitigation:** move them into a shared crate repo or workspace later, so looking_glass gains MCP and skills too.

## Consequences

- The tool surface (`cruncha-mcp`) and the skills become the agent-facing contract, so we should version them.
- Every agent action is mediated by the engine, which keeps provenance and the single-writer rule intact.
- Same owner, so code can move between looking_glass and Cruncha freely. The MIT notice is kept in Cruncha's copy.

## Action Items

1. [x] `cruncha-mcp` with read-only tools first (2026-11-20):
   - Tools: `read_doc` (the text mirror), `run` (current results, waiting for Python), `explain`, `check`, `history`.
   - Built on `rmcp` 3.11 (Apache-2.0), Streamable HTTP, served by the app on 127.0.0.1 with a bearer token, behind a Host check and refusal of browser `Origin`s.
   - The user turns it on in the Agents dialog, which shows the `claude mcp add` command.
   - The crate doesn't depend on `cruncha-store`; the app implements its `Backend` trait.
2. [ ] Try the interim setup: looking_glass + Claude Code + `cruncha-mcp`. Check that the external ACP adapters actually honor `mcpServers` and project skills (unverified).
3. [x] Port `lg-acp` with the `mcpServers` and capability changes (2026-11-20):
   - `crates/cruncha-acp` keeps the MIT notice in `LICENSE-LOOKING-GLASS`.
   - The client advertises no fs or terminal; there are no remote agents.
   - Sessions get `cruncha` as an HTTP MCP server. Agents that don't support HTTP MCP get an explicit error (a stdio bridge is a follow-up). *Follow-up done 2026-11-21:* they get it as a stdio server instead, `cruncha-app --mcp-stdio <url>` (the token in the bridge's environment); the error remains only when no bridge is available.
   - Tested against looking_glass's `lg-agent` in demo mode.
   - The in-app **agent panel** follows looking_glass's session view:
     - agent picker (Claude Code, Codex, Gemini CLI, Qwen Code through their `npx` adapters; looking_glass's agent when installed);
     - streaming Markdown (markdown-it, no raw HTML) with cell IDs linked to the document;
     - collapsible thinking, tool cards, plan;
     - permission prompts, marked when a tool acts outside the document;
     - the agent's model and mode menus, a usage pill;
     - saved conversations per document.
   - Each agent gets its own MCP server and token. It works in `agents/<doc id>/` in the app's data, which holds the skills (`.claude/skills/…`, `AGENTS.md`, `CLAUDE.md`).
4. [x] lg-agent: MCP client (check the license of an MCP crate such as `rmcp`), skills loader, per-session cancellation. Done 2026-11-21 as a port, `crates/cruncha-agent` (MIT notice kept), so looking_glass is unchanged:
   - a small Streamable-HTTP MCP client on `ureq` (no second async runtime), JSON and SSE answers;
   - its tools are the session's MCP tools plus `load_skill`; the coding tools are gone;
   - the system prompt is Cruncha's rules, `AGENTS.md` and the skills index; skill bodies load on demand;
   - cancellation is per session; the demo mode reads the document;
   - the app runs it as `cruncha-app --acp-agent`, listed as "Cruncha agent";
   - tests drive it through `cruncha-acp` against a real `cruncha-mcp` server (demo, OpenAI-style and Claude loops).
5. [ ] Consider upstreaming MCP and skills support to looking_glass.
