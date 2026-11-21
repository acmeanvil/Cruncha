<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Agents' proposals (ADR-0016): tracked-change cards with the source diff and the engine's
     "if accepted" results. Accepting is one commit by the user; rejecting keeps the record. -->
<script lang="ts">
  import type { CellResult, ProposalChange, ProposalView } from "./api";
  import { date } from "./format";

  let {
    proposals,
    editable,
    onaccept,
    onreject,
    onjumpcell,
  }: {
    proposals: ProposalView[];
    /** The latest version is shown (proposals can be decided). */
    editable: boolean;
    onaccept: (id: string, force: boolean) => Promise<void>;
    onreject: (id: string, note: string) => Promise<void>;
    onjumpcell: (id: string) => void;
  } = $props();

  let showDecided = $state(false);
  let rejecting = $state<string | null>(null);
  let note = $state("");
  let busy = $state<string | null>(null);

  const open = $derived(proposals.filter((p) => p.status === "open"));
  const decided = $derived(proposals.filter((p) => p.status !== "open"));

  const agentName = (actor: string) => actor.replace(/^ai:/, "");

  /** A prose cell's ProseMirror JSON as its text; other sources as they are. */
  function shown(kind: string, src: string | null): string {
    if (src == null) return "";
    if (kind !== "prose") return src;
    try {
      const node = JSON.parse(src);
      const text = (node.content ?? [])
        .map((n: { type: string; text?: string }) => (n.type === "hard_break" ? "\n" : (n.text ?? "")))
        .join("");
      return node.type === "heading" ? `${"#".repeat(node.attrs?.level ?? 1)} ${text}` : text;
    } catch {
      return src;
    }
  }

  function line(r: CellResult | null, none: string): string {
    if (!r) return none;
    if (r.error) return `Error: ${r.error}`;
    if (r.python && r.python.state !== "done") return `Python: ${r.python.state.replace(/_/g, " ")}${r.python.note ? ` (${r.python.note})` : ""}`;
    if (r.pending) return "not calculated yet (waiting for Python)";
    return r.text || "(no result)";
  }

  const verb = (c: ProposalChange) => ({ insert: "Add", modify: "Change", delete: "Delete" })[c.op];

  async function act(id: string, f: () => Promise<void>) {
    busy = id;
    try {
      await f();
    } finally {
      busy = null;
    }
  }
</script>

<div class="panel">
  {#if open.length === 0}
    <p class="empty small muted">
      No open proposals. When an agent suggests a change, it appears here for you to accept or reject; nothing
      changes in the document until you accept.
    </p>
  {/if}
  {#each open as p (p.id)}
    <article class="card" class:stale={p.stale}>
      <header>
        <span class="who">{agentName(p.proposedBy)}</span>
        <span class="muted small">{date(p.createdAtMs)} · against v{p.baseVersion}</span>
      </header>
      <p class="message">{p.message}</p>
      {#if p.stale}
        <p class="warn small">
          Some of these cells changed after the agent read them. Compare with the current source below, then
          reject it (and ask the agent to redo it) or accept it anyway, which replaces the current source.
        </p>
      {/if}
      {#each p.changes as c (c.id + c.op)}
        <section class="change" class:stale={c.stale}>
          <div class="what">
            <span class="op {c.op}">{verb(c)}</span>
            <span class="kind">{c.kind}</span>
            {#if c.op === "insert"}
              <span class="muted small">{c.after ? "after" : "at the start"}</span>
              {#if c.after}<button class="id" onclick={() => onjumpcell(c.after!)}>{c.after.slice(-6)}</button>{/if}
            {:else}
              <button class="id" onclick={() => onjumpcell(c.id)} title={c.id}>{c.id.slice(-6)}</button>
            {/if}
            {#if c.stale}<span class="badge">changed since</span>{/if}
          </div>
          {#if c.op !== "insert" && c.before != null && (c.op === "delete" || c.source != null)}
            <pre class="src del">{shown(c.kind, c.before)}</pre>
          {/if}
          {#if c.source != null}
            <pre class="src add">{shown(c.kind, c.source)}</pre>
          {/if}
          {#if c.description != null && c.description !== c.descriptionBefore}
            <p class="small desc">
              Description: {#if c.descriptionBefore}<del>{c.descriptionBefore}</del> → {/if}<ins>{c.description}</ins>
            </p>
          {/if}
        </section>
      {/each}
      {#if p.effects.length}
        <section class="effects">
          <h4>If accepted</h4>
          {#each p.effects as e (e.id)}
            <div class="effect">
              <button class="id" onclick={() => onjumpcell(e.id)} title={e.id}>{e.id.slice(-6)}</button>
              <div class="lines">
                <div class="now muted">{line(e.before, "new cell")}</div>
                <div class="then" class:err={e.after?.error} class:fails={e.after?.check === false}>
                  → {line(e.after, "deleted")}{#if e.after?.check === false} ✗ check fails{:else if e.after?.check === true} ✓{/if}
                </div>
              </div>
            </div>
          {/each}
        </section>
      {/if}
      {#if editable}
        {#if rejecting === p.id}
          <form
            class="reject"
            onsubmit={(ev) => {
              ev.preventDefault();
              void act(p.id, async () => {
                await onreject(p.id, note);
                rejecting = null;
                note = "";
              });
            }}
          >
            <!-- svelte-ignore a11y_autofocus -->
            <input bind:value={note} placeholder="Why? (optional; the agent can read it)" autofocus />
            <button type="submit" class="danger" disabled={busy === p.id}>Reject</button>
            <button type="button" class="link" onclick={() => (rejecting = null)}>Cancel</button>
          </form>
        {:else}
          <div class="actions">
            {#if p.stale}
              <button disabled={busy === p.id} onclick={() => void act(p.id, () => onaccept(p.id, true))}>Accept anyway</button>
            {:else}
              <button class="primary" disabled={busy === p.id} onclick={() => void act(p.id, () => onaccept(p.id, false))}>Accept</button>
            {/if}
            <button disabled={busy === p.id} onclick={() => ((rejecting = p.id), (note = ""))}>Reject…</button>
          </div>
        {/if}
      {:else}
        <p class="muted small">Go back to the latest version to decide.</p>
      {/if}
    </article>
  {/each}

  {#if decided.length}
    <button class="link small toggle" onclick={() => (showDecided = !showDecided)}>
      {showDecided ? "Hide" : "Show"} {decided.length} decided
    </button>
    {#if showDecided}
      {#each decided as p (p.id)}
        <article class="card decided">
          <header>
            <span class="status {p.status}">{p.status}</span>
            <span class="who">{agentName(p.proposedBy)}</span>
            <span class="muted small">{p.decidedAtMs ? date(p.decidedAtMs) : ""}</span>
          </header>
          <p class="message">{p.message}</p>
          <p class="muted small">
            {p.changes.length} {p.changes.length === 1 ? "change" : "changes"}{#if p.acceptedVersion != null} · version {p.acceptedVersion}{/if}{#if p.decidedBy} · by {p.decidedBy.replace(/^(user|ai):/, "")}{/if}
          </p>
          {#if p.decisionNote}<p class="small">Note: {p.decisionNote}</p>{/if}
        </article>
      {/each}
    {/if}
  {/if}
</div>

<style>
  .panel { padding: 8px; display: grid; gap: 8px; align-content: start; }
  .empty { padding: 8px; }
  .card {
    border: 1px solid var(--border); border-left: 3px solid var(--accent); border-radius: 6px;
    padding: 8px 10px; background: var(--panel); display: grid; gap: 6px;
  }
  .card.stale { border-left-color: var(--warn); }
  .card.decided { border-left-color: var(--border); opacity: 0.85; }
  header { display: flex; gap: 6px; align-items: baseline; flex-wrap: wrap; }
  .who { font-weight: 600; }
  .message { margin: 0; white-space: pre-wrap; }
  .warn { margin: 0; color: var(--warn); background: var(--warn-bg); border-radius: 4px; padding: 4px 6px; }
  .change { display: grid; gap: 3px; }
  .what { display: flex; gap: 6px; align-items: center; flex-wrap: wrap; }
  .op { font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.03em; }
  .op.insert { color: var(--echo); }
  .op.modify { color: var(--accent); }
  .op.delete { color: var(--danger); }
  .kind { font-size: 12px; color: var(--muted); }
  .id {
    border: none; background: var(--sunken); border-radius: 4px; padding: 0 5px;
    font: 11.5px var(--mono); color: var(--text);
  }
  .badge { font-size: 11px; color: var(--warn); background: var(--warn-bg); border-radius: 8px; padding: 0 6px; }
  .src {
    margin: 0; font: 12px var(--mono); white-space: pre-wrap; word-break: break-word;
    padding: 4px 6px; border-radius: 4px; max-height: 14em; overflow: auto;
  }
  .src.del { background: var(--danger-bg); text-decoration: line-through; text-decoration-color: color-mix(in srgb, var(--danger) 50%, transparent); }
  .src.add { background: var(--accent-bg); }
  .desc { margin: 0; }
  .effects { border-top: 1px solid var(--border); padding-top: 6px; display: grid; gap: 4px; }
  h4 { margin: 0; font-size: 12px; color: var(--muted); font-weight: 600; text-transform: uppercase; letter-spacing: 0.03em; }
  .effect { display: grid; grid-template-columns: auto 1fr; gap: 6px; align-items: start; }
  .lines { font: 12px var(--mono); display: grid; gap: 1px; min-width: 0; }
  .lines > div { overflow-wrap: anywhere; }
  .then.err, .then.fails { color: var(--danger); }
  .actions, .reject { display: flex; gap: 6px; align-items: center; }
  .reject input { flex: 1; min-width: 0; }
  .toggle { justify-self: start; margin: 4px 2px; }
  .status { font-size: 11px; font-weight: 600; text-transform: uppercase; color: var(--muted); }
  .status.accepted { color: var(--echo); }
  .status.rejected { color: var(--danger); }
</style>
