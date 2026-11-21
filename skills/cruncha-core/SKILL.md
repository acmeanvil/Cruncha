---
name: cruncha-core
description: Ground rules for any AI work in a Cruncha calculation document. Load it before reading, editing, explaining or checking a Cruncha document. It covers that the engine computes every number and the agent never does, that agent edits are proposals a human accepts, the document tools, cell IDs and provenance.
---

# Working in a Cruncha document

Cruncha documents are engineering calculation packages: prose, calc cells, Python and Rust code cells, data and plots in one reactive graph. Engineers sign them and submit them to clients and authorities. Everything below follows from that.

## Non-negotiable rules

1. **Never state a computed number that the engine didn't produce.** If you need a value, write or locate a cell and run it, then quote its result with its cell ID. Mental arithmetic, estimates and "approximately" values must be labeled as such, and never written into the document as results.
2. **Your edits are proposals.** `propose_edit` records a change that a human accepts or rejects in the app; the document doesn't change until they accept. Don't describe a proposal as done or applied. Accepted cells carry provenance `ai:accepted`.
3. **Don't silently change inputs.** Changing a design input (load, span, material grade, code factor) changes the engineering. Propose it separately, with the reason, and never bundle it with formatting edits.
4. **Units always.** Every physical quantity gets a unit. Never strip units to "make it work"; fix the dimensional problem instead.
5. **Cite sources for design values** (code clause, table, manufacturer data) in the prose next to the cell. If you don't know the source, say so and mark the value as a placeholder (`_` suffix) so a human fills it in.
6. **One name, one definition.** Each name is defined by exactly one cell in the document. Use `read_doc` to check that a name is free before defining it.

## Tools

| Tool | Purpose |
| --- | --- |
| `read_doc(range?)` | The document as Markdown mirror text, with cell IDs and current results (`⇒` lines). Start here. |
| `run(cell_ids?)` | Evaluate the given cells (or everything stale) and return results and errors |
| `propose_edit(changes, message, base_version?)` | Insert, modify or delete cells as one proposal; returns its ID, the new cells' IDs, and the "if accepted" results |
| `proposals()` | Open proposals (any agent's) and your decided ones, with the reviewer's notes |
| `withdraw(proposal_id)` | Take back one of your open proposals |
| `explain(cell_id)` | Dependencies, echo with substituted values, and the definitions of everything the cell reads |
| `check(scope?)` | Built-in checks: unit consistency, names used before they're defined, unfilled placeholders, failing `@test`s, cycles |
| `history(cell_id?, since?)` | Versions, actors and diffs (see the `cruncha-history` skill) |
| `query_data(name, polars_or_sql)` | Read-only query on a data frame in the document |

**Available now:** `read_doc(from?, limit?)`, `run(cell_ids?)`, `explain(cell_id)`, `check()`, `history(cell_id?, since?)`, `propose_edit`, `proposals()` and `withdraw(proposal_id)`.
- `run` returns the engine's current results; the document recalculates by itself, and `run` waits for Python cells that are still running.
- `query_data` isn't available yet.

### `propose_edit`

`changes` is a list, applied in order as one unit (ADR-0016):

```json
[
  {"op": "modify", "id": "01JC3…", "source": "L = 20 ft"},
  {"op": "insert", "after": "01JC4…", "kind": "calc", "source": "M_half = M / 2 = ?? kip·ft", "description": "Half the midspan moment"},
  {"op": "delete", "id": "01JC5…"}
]
```

- `insert`: `kind` is `calc`, `python` or `prose`; `after` is the cell it follows (omit it for the start of the document). Prose is one paragraph, or a heading written `## Title`. Several new cells in a row: list them in reverse order with the same `after`, or propose the first, then use the ID the reply gives it.
- `modify`: a new `source`, a new `description`, or both. Other attributes are kept.
- Pass `base_version`, the version in `read_doc`'s header. A cell changed after it is refused, so you re-read instead of overwriting someone's edit.
- The engine checks the combined document. A proposal that would define a name twice or make a cycle is refused, with the reason.
- The reply lists every cell whose result would change, **now** and **if accepted**. Quote those as "if accepted", never as the document's results.
- Python cells you add or change don't run until the user accepts and runs them, so their results show as not calculated.
- The user sees a card with the source diff and the results, and markers in the document. They accept or reject the whole proposal. If they edit a cell first, the proposal goes stale; they can still accept it, or reject it and ask you to redo it.
- A rejection can carry a note; read it with `proposals()` before trying again.

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
4. Report the "if accepted" results from the reply, by cell. After the user accepts, `run` (or `read_doc`) gives the document's results.

For calc syntax, see `cruncha-calc-authoring`. For code cells, see `cruncha-code-cells`.
