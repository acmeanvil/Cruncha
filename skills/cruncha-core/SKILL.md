---
name: cruncha-core
description: Ground rules for any AI work in a Cruncha calculation document. Load it before reading, editing, explaining or checking a Cruncha document. It covers that the engine computes every number and the agent never does, that agent edits are proposals a human accepts, the document tools, cell IDs and provenance.
---

# Working in a Cruncha document

Cruncha documents are engineering calculation packages: prose, calc cells, Python and Rust code cells, data and plots in one reactive graph. Engineers sign them and submit them to clients and authorities. Everything below follows from that.

## Non-negotiable rules

1. **Never state a computed number that the engine didn't produce.** If you need a value, write or locate a cell and run it, then quote its result with its cell ID. Mental arithmetic, estimates and "approximately" values must be labeled as such, and never written into the document as results.
2. **Your edits are proposals.** `propose_edit` creates changes marked `ai:proposed`. They show as tracked changes, and a human accepts or rejects them. Don't describe a proposal as done or applied.
3. **Don't silently change inputs.** Changing a design input (load, span, material grade, code factor) changes the engineering. Propose it separately, with the reason, and never bundle it with formatting edits.
4. **Units always.** Every physical quantity gets a unit. Never strip units to "make it work"; fix the dimensional problem instead.
5. **Cite sources for design values** (code clause, table, manufacturer data) in the prose next to the cell. If you don't know the source, say so and mark the value as a placeholder (`_` suffix) so a human fills it in.
6. **One name, one definition.** Each name is defined by exactly one cell in the document. Use `read_doc` to check that a name is free before defining it.

## Tools

| Tool | Purpose |
| --- | --- |
| `read_doc(range?)` | The document as Markdown mirror text, with cell IDs and current results (`⇒` lines). Start here. |
| `run(cell_ids?)` | Evaluate the given cells (or everything stale) and return results and errors |
| `propose_edit(changes, message)` | Insert, modify or delete cells as one proposal; returns a proposal ID |
| `explain(cell_id)` | Dependencies, echo with substituted values, and the definitions of everything the cell reads |
| `check(scope?)` | Built-in checks: unit consistency, names used before they're defined, unfilled placeholders, failing `@test`s, cycles |
| `history(cell_id?, since?)` | Versions, actors and diffs (see the `cruncha-history` skill) |
| `query_data(name, polars_or_sql)` | Read-only query on a data frame in the document |

**Available now** (ADR-0009, phase 1): `read_doc(from?, limit?)`, `run(cell_ids?)`, `explain(cell_id)`, `check()`, and `history(cell_id?, since?)`.
- `run` returns the engine's current results; the document recalculates by itself, and `run` waits for Python cells that are still running.
- `propose_edit` and `query_data` aren't available yet. Until they are, describe a change in words and let the user make it.

**Inside Cruncha:** the user opens the **Agent** panel and starts an agent (Claude Code, Codex, Gemini CLI, …). The `cruncha` tools are already connected. Your working folder only holds these instructions: don't create files or run commands there.

**Outside Cruncha:** the user turns on access in **Agent → Use an agent outside Cruncha…**. That starts an MCP server over HTTP on 127.0.0.1, with a bearer token; the dialog shows the `claude mcp add …` command and a JSON block for other agents. It serves whichever document is open in the app. If a tool says no document is open, ask the user to open one.

In `read_doc`'s mirror, each cell's fence carries its ID (```` ```calc {#01JC3… description="…"} ````), and the engine's result follows on a `> ⇒` line. A failing check is marked `✗ (CHECK FAILS)`. A cell that wasn't computed says `not calculated` and why.

## Cell IDs and provenance

- Every cell has a stable ID (a ULID such as `01JC3…`). Refer to cells by ID in tool calls and by name in prose ("the moment `M_u` in cell 01JC3…").
- Provenance is one of `human`, `ai:proposed`, `ai:accepted`, or `imported`. When you're asked to summarize a document, report how many results rest on cells still `ai:proposed`.

## Workflow

1. `read_doc`, and `check` if you're editing.
2. Plan the change. For anything beyond a typo, tell the user what you'll propose before proposing it.
3. Make one coherent `propose_edit` per logical change, with a message that a reviewer can read.
4. `run` the affected cells and report the results by quoting the cells, plus any errors.

For calc syntax, see `cruncha-calc-authoring`. For code cells, see `cruncha-code-cells`.
