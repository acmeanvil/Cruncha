<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The left rail: the document outline and history. Variables and proposals are in the bottom
     panel (BottomPanel.svelte). -->
<script lang="ts" module>
  export type SidebarTab = "outline" | "history";
</script>

<script lang="ts">
  import type { RefView, Selection, VersionView } from "./api";
  import type { OutlineItem } from "./editor/outline";
  import HistoryPanel from "./HistoryPanel.svelte";

  let {
    outline,
    history,
    refs,
    latest,
    earliest,
    selected,
    onselect,
    onjump,
    onsnapshot,
    tab = $bindable("outline"),
  }: {
    outline: OutlineItem[];
    history: VersionView[];
    refs: RefView[];
    latest: number;
    earliest: number;
    selected: Selection;
    onselect: (s: Selection) => void;
    onjump: (pos: number) => void;
    onsnapshot: () => void;
    tab?: SidebarTab;
  } = $props();

  const TABS: { id: SidebarTab; label: string; title: string }[] = [
    { id: "outline", label: "Outline", title: "Headings and what each section defines" },
    { id: "history", label: "History", title: "Versions and snapshots" },
  ];
  const edits = $derived(history.filter((v) => v.kind !== "maintenance").length);
</script>

<aside>
  <div class="tabs" role="tablist">
    {#each TABS as t (t.id)}
      {@const n = t.id === "history" ? edits : 0}
      <button
        role="tab"
        aria-selected={tab === t.id}
        class:on={tab === t.id}
        title={t.title}
        onclick={() => (tab = t.id)}
      >
        <span class="icon" aria-hidden="true">
          <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round">
            {#if t.id === "outline"}
              <path d="M2.5 3.5h11M4.5 6.5h9M4.5 9.5h9M2.5 12.5h11" />
            {:else}
              <circle cx="8" cy="8" r="5.5" /><path d="M8 5v3l2 1.5" />
            {/if}
          </svg>
          {#if n > 0}<span class="count">{n > 99 ? "99+" : n}</span>{/if}
        </span>
        <span class="label">{t.label}</span>
      </button>
    {/each}
  </div>
  <div class="body">
    {#if tab === "outline"}
      {#if outline.length === 0}
        <p class="empty small muted">Headings appear here. Type <kbd>#</kbd> and a space to start one.</p>
      {:else}
        <nav>
          {#each outline as item (item.pos)}
            <button class="item l{item.level}" onclick={() => onjump(item.pos)}>
              <span class="text">{item.text || "Untitled heading"}</span>
              {#if item.defines?.length}
                <span class="defs">{item.defines.join(", ")}</span>
              {/if}
            </button>
          {/each}
        </nav>
      {/if}
    {:else}
      <HistoryPanel {history} {refs} {latest} {earliest} {selected} {onselect} {onsnapshot} />
    {/if}
  </div>
</aside>

<style>
  aside {
    display: grid; grid-template-rows: auto 1fr; min-height: 0;
    border-right: 1px solid var(--border); background: var(--panel);
  }
  .tabs { display: grid; grid-template-columns: repeat(2, 1fr); padding: 6px 6px 0; border-bottom: 1px solid var(--border); }
  .tabs button {
    border: none; background: none; border-radius: 6px 6px 0 0; padding: 5px 2px 4px;
    color: var(--muted); border-bottom: 2px solid transparent; margin-bottom: -1px;
    display: flex; justify-content: center; align-items: center; gap: 6px; min-width: 0;
  }
  .tabs button:hover { color: var(--text); background: var(--sunken); }
  .tabs button.on { color: var(--text); border-bottom-color: var(--accent); }
  .tabs button.on .label { font-weight: 600; }
  .icon { position: relative; display: block; height: 16px; }
  .label { font-size: 12px; line-height: 1.2; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 100%; }
  .count {
    position: absolute; left: 12px; top: -5px; font-size: 9.5px; line-height: 13px; min-width: 13px; padding: 0 3px;
    background: var(--sunken); color: var(--muted); border: 1px solid var(--border); border-radius: 7px; text-align: center;
  }
  .body { overflow: auto; min-height: 0; }
  nav { padding: 8px 6px; display: grid; }
  .item {
    border: none; background: none; text-align: left; padding: 4px 8px; border-radius: 5px;
    display: grid; gap: 0;
  }
  .item:hover { background: var(--sunken); }
  .item.l1 { font-weight: 600; }
  .item.l2 { padding-left: 20px; }
  .item.l3 { padding-left: 32px; font-size: 13px; }
  .item:is(.l4, .l5, .l6) { padding-left: 44px; font-size: 13px; color: var(--muted); }
  .text { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .defs {
    font: 11.5px var(--mono); color: var(--muted); overflow: hidden; text-overflow: ellipsis;
    white-space: nowrap;
  }
  .empty { padding: 16px; }
  kbd {
    font: 11px var(--mono); border: 1px solid var(--border); border-radius: 3px; padding: 0 4px;
    background: var(--sunken);
  }
</style>
