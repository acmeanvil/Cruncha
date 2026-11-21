<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The left rail: the document outline, variables, agents' proposals and history. -->
<script lang="ts" module>
  export type SidebarTab = "outline" | "variables" | "proposals" | "history";
</script>

<script lang="ts">
  import type { ProposalView, RefView, Selection, VersionView } from "./api";
  import type { OutlineItem } from "./editor/outline";
  import type { Variable } from "./editor/variables";
  import HistoryPanel from "./HistoryPanel.svelte";
  import ProposalsPanel from "./ProposalsPanel.svelte";
  import VariableBrowser from "./VariableBrowser.svelte";

  let {
    outline,
    variables,
    editable,
    history,
    refs,
    latest,
    earliest,
    selected,
    onselect,
    onjump,
    onjumpcell,
    ondescribe,
    onsnapshot,
    proposals,
    onaccept,
    onreject,
    tab = $bindable("outline"),
  }: {
    outline: OutlineItem[];
    variables: Variable[];
    editable: boolean;
    history: VersionView[];
    refs: RefView[];
    latest: number;
    earliest: number;
    selected: Selection;
    onselect: (s: Selection) => void;
    onjump: (pos: number) => void;
    onjumpcell: (id: string) => void;
    ondescribe: (id: string, description: string) => void;
    onsnapshot: () => void;
    proposals: ProposalView[];
    onaccept: (id: string, force: boolean) => Promise<void>;
    onreject: (id: string, note: string) => Promise<void>;
    tab?: SidebarTab;
  } = $props();

  const TABS: { id: SidebarTab; label: string; title: string }[] = [
    { id: "outline", label: "Outline", title: "Headings and what each section defines" },
    { id: "variables", label: "Variables", title: "Every defined name with its value" },
    { id: "proposals", label: "Proposals", title: "Changes agents proposed, to accept or reject" },
    { id: "history", label: "History", title: "Versions and snapshots" },
  ];
  const openProposals = $derived(proposals.filter((p) => p.status === "open").length);
  const edits = $derived(history.filter((v) => v.kind !== "maintenance").length);
</script>

<aside>
  <div class="tabs" role="tablist">
    {#each TABS as t (t.id)}
      {@const n = t.id === "variables" ? variables.length : t.id === "proposals" ? openProposals : t.id === "history" ? edits : 0}
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
            {:else if t.id === "variables"}
              <path d="M5 2.5c-1.5 0-1.5 1-1.5 2.5S3 7.5 2 8c1 .5 1.5 1.5 1.5 3s0 2.5 1.5 2.5M11 2.5c1.5 0 1.5 1 1.5 2.5S13 7.5 14 8c-1 .5-1.5 1.5-1.5 3s0 2.5-1.5 2.5M6.5 6l3 4M9.5 6l-3 4" />
            {:else if t.id === "proposals"}
              <path d="M9.5 2.5l4 4-6.5 6.5H3v-4zM8 4l4 4" />
            {:else}
              <circle cx="8" cy="8" r="5.5" /><path d="M8 5v3l2 1.5" />
            {/if}
          </svg>
          {#if n > 0}<span class="count" class:due={t.id === "proposals"}>{n > 99 ? "99+" : n}</span>{/if}
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
    {:else if tab === "variables"}
      <VariableBrowser {variables} {editable} onjump={onjumpcell} {ondescribe} />
    {:else if tab === "proposals"}
      <ProposalsPanel {proposals} {editable} {onaccept} {onreject} {onjumpcell} />
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
  .tabs { display: grid; grid-template-columns: repeat(4, 1fr); padding: 6px 6px 0; border-bottom: 1px solid var(--border); }
  .tabs button {
    border: none; background: none; border-radius: 6px 6px 0 0; padding: 5px 2px 4px;
    color: var(--muted); border-bottom: 2px solid transparent; margin-bottom: -1px;
    display: grid; justify-items: center; gap: 2px; min-width: 0;
  }
  .tabs button:hover { color: var(--text); background: var(--sunken); }
  .tabs button.on { color: var(--text); border-bottom-color: var(--accent); }
  .tabs button.on .label { font-weight: 600; }
  .icon { position: relative; display: block; height: 16px; }
  .label { font-size: 11px; line-height: 1.2; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 100%; }
  .count {
    position: absolute; left: 12px; top: -5px; font-size: 9.5px; line-height: 13px; min-width: 13px; padding: 0 3px;
    background: var(--sunken); color: var(--muted); border: 1px solid var(--border); border-radius: 7px; text-align: center;
  }
  .count.due { background: var(--accent); border-color: var(--accent); color: #fff; }
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
