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

  const openProposals = $derived(proposals.filter((p) => p.status === "open").length);
  const edits = $derived(history.filter((v) => v.kind !== "maintenance").length);
</script>

<aside>
  <div class="tabs" role="tablist">
    <button role="tab" aria-selected={tab === "outline"} class:on={tab === "outline"} onclick={() => (tab = "outline")}>
      Outline
    </button>
    <button role="tab" aria-selected={tab === "variables"} class:on={tab === "variables"} onclick={() => (tab = "variables")}>
      Variables <span class="count">{variables.length}</span>
    </button>
    <button role="tab" aria-selected={tab === "proposals"} class:on={tab === "proposals"} onclick={() => (tab = "proposals")}>
      Proposals {#if openProposals}<span class="count due">{openProposals}</span>{/if}
    </button>
    <button role="tab" aria-selected={tab === "history"} class:on={tab === "history"} onclick={() => (tab = "history")}>
      History <span class="count">{edits}</span>
    </button>
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
  .tabs { display: flex; gap: 0; overflow-x: auto; padding: 8px 10px 0; border-bottom: 1px solid var(--border); }
  .tabs button {
    border: none; background: none; border-radius: 6px 6px 0 0; padding: 6px 8px;
    color: var(--muted); border-bottom: 2px solid transparent; margin-bottom: -1px;
  }
  .tabs button.on { color: var(--text); border-bottom-color: var(--accent); font-weight: 600; }
  .count {
    font-size: 11px; background: var(--sunken); border-radius: 8px; padding: 0 6px; margin-left: 2px;
    font-weight: normal;
  }
  .count.due { background: var(--accent); color: #fff; }
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
