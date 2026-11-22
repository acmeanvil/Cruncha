<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The archive (ADR-0023): snapshots whose groups were archived (kept, opened read-only from
     here, restorable and printable like any snapshot) and the save log, the record of every
     save that archiving or trimming removed. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { api, type ArchivedSave } from "./api";
  import { date } from "./format";
  import type { Group } from "./history";
  import Modal from "./Modal.svelte";

  let {
    groups,
    onclose,
    onopen,
  }: {
    /** Archived groups, newest first (history.ts `splitArchived`). */
    groups: Group[];
    onclose: () => void;
    /** Open a snapshot read-only in the document view. */
    onopen: (name: string) => void;
  } = $props();

  let log = $state<ArchivedSave[] | null>(null);
  let failed = $state<string | null>(null);
  let query = $state("");
  const PAGE = 100;
  let limit = $state(PAGE);

  onMount(() => {
    api.archivedSaves().then(
      (l) => (log = l.reverse()),
      (e) => (failed = String(e)),
    );
  });

  const snapshots = $derived(groups.filter((g) => g.snapshot).map((g) => g.snapshot!));
  const matches = $derived.by(() => {
    const q = query.trim().toLowerCase().replace(/^v(?=\d)/, "");
    if (!log) return [];
    if (!q) return log;
    return log.filter(
      (s) =>
        String(s.version) === q ||
        s.message.toLowerCase().includes(q) ||
        s.actor.toLowerCase().includes(q) ||
        (s.snapshot ?? "draft").toLowerCase().includes(q) ||
        s.cells.some((c) => c.toLowerCase() === q),
    );
  });

  const kinds: Record<string, string> = { create: "created", restore: "restore", proposal: "accepted proposal" };
</script>

<Modal title="Archive" {onclose} width={620}>
  <h3>Snapshots</h3>
  {#if snapshots.length === 0}
    <p class="small muted">No snapshots were archived (only draft saves).</p>
  {:else}
    <ul class="snaps">
      {#each snapshots as r (r.name)}
        <li>
          <span class="tag">{r.name}</span>
          <span class="small muted">v{r.version} · {date(r.createdAtMs)}{r.message ? ` · ${r.message}` : ""}</span>
          <span class="spacer"></span>
          <button class="small" onclick={() => onopen(r.name)} title="Show it read-only; from there you can restore it or print a report">Open</button>
        </li>
      {/each}
    </ul>
  {/if}

  <h3>Save log</h3>
  <p class="small muted">Every save that archiving or trimming removed: who, when and which cells it changed.</p>
  <input type="search" placeholder="Find by message, person, snapshot, version or cell ID" aria-label="Find a save" bind:value={query} />
  {#if failed}
    <p class="small err">The save log didn't load: {failed}</p>
  {:else if !log}
    <p class="small muted">Loading…</p>
  {:else if matches.length === 0}
    <p class="small muted">{log.length ? `No save matches “${query}”.` : "No saves were removed yet."}</p>
  {:else}
    <table class="small">
      <thead><tr><th>Version</th><th>When</th><th>Who</th><th>What</th><th>Cells</th></tr></thead>
      <tbody>
        {#each matches.slice(0, limit) as s (s.version)}
          <tr>
            <td class="mono">v{s.version}<span class="muted"> {s.snapshot ?? "Draft"}</span></td>
            <td>{date(s.atMs)}</td>
            <td>{s.actor}</td>
            <td>{s.message || kinds[s.kind] || s.kind}{s.kind in kinds && s.message ? ` (${kinds[s.kind]})` : ""}</td>
            <td title={s.cells.join("\n")}>{s.cells.length}</td>
          </tr>
        {/each}
      </tbody>
    </table>
    {#if matches.length > limit}
      <button class="link small" onclick={() => (limit += PAGE * 5)}>Show more ({matches.length - limit} left)</button>
    {/if}
  {/if}
</Modal>

<style>
  h3 { font-size: 12px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--muted); margin: 12px 0 6px; }
  h3:first-child { margin-top: 0; }
  .snaps { list-style: none; padding: 0; margin: 0; }
  .snaps li { display: flex; gap: 8px; align-items: baseline; padding: 4px 0; border-bottom: 1px solid var(--border); }
  .spacer { flex: 1; }
  .tag { background: var(--accent-bg); color: var(--accent); border-radius: 4px; padding: 0 6px; font-weight: 600; font-size: 12px; }
  input[type="search"] { width: 100%; margin: 4px 0 6px; }
  table { width: 100%; border-collapse: collapse; }
  th { text-align: left; color: var(--muted); font-weight: 500; border-bottom: 1px solid var(--border); padding: 3px 6px; }
  td { padding: 3px 6px; border-bottom: 1px solid color-mix(in srgb, var(--border) 60%, transparent); vertical-align: top; }
  .mono { font-family: var(--mono); white-space: nowrap; }
  .err { color: var(--danger); }
</style>
