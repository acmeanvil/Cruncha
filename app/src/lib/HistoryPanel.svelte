<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Version history and snapshots (named, known-good versions; store tags that survive history
     trimming). Selecting one shows the document as it was. -->
<script lang="ts">
  import type { RefView, Selection, VersionView } from "./api";
  import { date } from "./format";

  let {
    history,
    refs,
    latest,
    earliest,
    selected,
    onselect,
    onsnapshot,
  }: {
    history: VersionView[];
    refs: RefView[];
    latest: number;
    earliest: number;
    selected: Selection;
    onselect: (s: Selection) => void;
    onsnapshot?: () => void;
  } = $props();

  /** Newest snapshot first, and how many edits came after the newest one. */
  const snapshots = $derived([...refs].sort((a, b) => b.version - a.version || b.createdAtMs - a.createdAtMs));
  const editsSince = $derived(
    snapshots.length
      ? history.filter((v) => v.version > snapshots[0].version && v.kind !== "maintenance").length
      : null,
  );

  let showMaintenance = $state(false);

  const shown = $derived(
    history.filter((v) => showMaintenance || v.kind !== "maintenance" || v.prunedBefore != null),
  );
  const tagsAt = $derived(
    refs.reduce<Record<number, string[]>>((m, r) => {
      (m[r.version] ??= []).push(r.name);
      return m;
    }, {}),
  );

  const isSelected = (v: VersionView) =>
    (selected.kind === "latest" && v.version === latest) ||
    (selected.kind === "version" && selected.version === v.version);
</script>

<div class="history">
  <section>
    <h3>
      Snapshots
      {#if onsnapshot}
        <button class="take" onclick={onsnapshot} title="Mark the latest version as a known-good point">+ Take snapshot</button>
      {/if}
    </h3>
    {#if snapshots.length === 0}
      <p class="small muted hint">
        A snapshot marks a known-good point, such as “Checked” or “Issued for review”. Snapshots are
        kept even when older history is trimmed, and you can restore one at any time.
      </p>
    {:else}
      {#if editsSince !== null}
        <p class="small muted since">
          {editsSince === 0 ? "No edits since" : `${editsSince} edit${editsSince === 1 ? "" : "s"} since`} “{snapshots[0].name}”
        </p>
      {/if}
      {#each snapshots as r (r.name)}
        <button
          class="row snap"
          class:sel={selected.kind === "ref" && selected.name === r.name}
          onclick={() => onselect({ kind: "ref", name: r.name })}
        >
          <span class="head">
            <span class="tag">{r.name}</span>
            <span class="muted small">v{r.version}{r.pinned ? " · kept copy" : ""}</span>
            <span class="muted small when">{date(r.createdAtMs)}</span>
          </span>
          {#if r.message}<span class="msg small">{r.message}</span>{/if}
        </button>
      {/each}
    {/if}
  </section>

  <section>
    <h3>
      History
      <label class="small muted toggle">
        <input type="checkbox" bind:checked={showMaintenance} /> maintenance
      </label>
    </h3>
    {#each shown as v (v.version)}
      {#if v.prunedBefore != null}
        <div class="marker small">History before version {v.prunedBefore} was trimmed</div>
      {:else}
        <button
          class="row"
          class:sel={isSelected(v)}
          class:maint={v.kind === "maintenance"}
          onclick={() =>
            onselect(v.version === latest ? { kind: "latest" } : { kind: "version", version: v.version })}
        >
          <span class="head">
            <span class="ver">v{v.version}</span>
            {#if v.kind === "restore"}<span class="badge">restore</span>{/if}
            {#each tagsAt[v.version] ?? [] as t}<span class="tag">{t}</span>{/each}
            <span class="muted small when">{date(v.timestampMs)}</span>
          </span>
          <span class="msg">{v.message || v.kind}</span>
          <span class="muted small">{v.actor}</span>
        </button>
      {/if}
    {/each}
    {#if earliest > 0}
      <p class="small muted end">Versions before {earliest} are no longer available.</p>
    {/if}
  </section>
</div>

<style>
  .history { padding: 4px 0; }
  section { padding: 0 10px 12px; }
  h3 {
    font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--muted);
    margin: 8px 4px; display: flex; justify-content: space-between; align-items: center;
  }
  .take {
    text-transform: none; letter-spacing: 0; font-size: 12px; padding: 2px 8px; color: var(--accent);
  }
  .hint, .since { margin: 0 4px 6px; }
  .row.snap { border-left: 3px solid var(--accent); }
  .toggle { text-transform: none; letter-spacing: 0; font-weight: normal; }
  .row {
    display: grid; gap: 1px; width: 100%; text-align: left; border: 1px solid transparent;
    background: none; border-radius: 6px; padding: 6px 8px; margin: 1px 0;
  }
  .row:hover { background: var(--sunken); }
  .row.sel { background: var(--select); border-color: var(--select-border); }
  .row.maint { opacity: 0.6; }
  .head { display: flex; gap: 6px; align-items: baseline; flex-wrap: wrap; }
  .ver { font-family: var(--mono); font-size: 12px; }
  .when { margin-left: auto; }
  .msg { overflow: hidden; text-overflow: ellipsis; }
  .tag {
    background: var(--accent-bg); color: var(--accent); border-radius: 4px;
    padding: 0 6px; font-size: 12px; font-weight: 600;
  }
  .badge { background: var(--sunken); border-radius: 4px; padding: 0 6px; font-size: 12px; }
  .marker {
    text-align: center; color: var(--muted); border-top: 1px dashed var(--border);
    border-bottom: 1px dashed var(--border); margin: 6px 0; padding: 4px;
  }
  .end { text-align: center; }
</style>
