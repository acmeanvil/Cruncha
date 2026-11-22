<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The panel along the bottom of the window: Variables (table and inspector) and agents'
     Proposals. Collapses to its tab strip; drag the top edge to resize (⌘J toggles it). Its
     height and open state are remembered on this computer. -->
<script lang="ts" module>
  export type BottomTab = "variables" | "proposals";
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import type { ProposalView } from "./api";
  import type { Variable } from "./editor/variables";
  import ProposalsPanel from "./ProposalsPanel.svelte";
  import VariableTable from "./VariableTable.svelte";

  let {
    variables,
    editable,
    proposals,
    onjumpcell,
    ondescribe,
    onaccept,
    onreject,
    tab = $bindable("variables"),
    open = $bindable(false),
  }: {
    variables: Variable[];
    editable: boolean;
    proposals: ProposalView[];
    onjumpcell: (id: string) => void;
    ondescribe: (id: string, description: string) => void;
    onaccept: (id: string, force: boolean) => Promise<void>;
    onreject: (id: string, note: string) => Promise<void>;
    tab?: BottomTab;
    open?: boolean;
  } = $props();

  const KEY = "cruncha.bottomPanel";
  const MIN = 120;
  let height = $state(260);
  let loaded = false;

  onMount(() => {
    try {
      const s = JSON.parse(localStorage.getItem(KEY) ?? "null");
      if (s && typeof s.height === "number") height = s.height;
      if (s && typeof s.open === "boolean" && !open) open = s.open;
      if (s && (s.tab === "variables" || s.tab === "proposals") && tab === "variables") tab = s.tab;
    } catch {
      /* no storage: defaults */
    }
    loaded = true;
  });

  $effect(() => {
    const s = JSON.stringify({ height, open, tab });
    if (!loaded) return;
    try {
      localStorage.setItem(KEY, s);
    } catch {
      /* ignore */
    }
  });

  const openProposals = $derived(proposals.filter((p) => p.status === "open").length);

  function pick(t: BottomTab) {
    if (open && tab === t) open = false;
    else {
      tab = t;
      open = true;
    }
  }

  function drag(e: PointerEvent) {
    e.preventDefault();
    const el = e.currentTarget as HTMLElement;
    el.setPointerCapture(e.pointerId);
    const y0 = e.clientY, h0 = height;
    const max = () => Math.max(MIN, window.innerHeight - 160);
    const move = (m: PointerEvent) => (height = Math.min(max(), Math.max(MIN, h0 + y0 - m.clientY)));
    const up = () => {
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerup", up);
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerup", up);
  }

  function nudge(e: KeyboardEvent) {
    if (e.key === "ArrowUp") height = Math.min(window.innerHeight - 160, height + 24);
    else if (e.key === "ArrowDown") height = Math.max(MIN, height - 24);
    else return;
    e.preventDefault();
  }
</script>

<section class="bottom" class:open style={open ? `height: ${height}px` : ""}>
  {#if open}
    <!-- A focusable separator is a window splitter (ARIA), which the arrow keys resize. -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
    <div
      class="grip"
      role="separator"
      aria-orientation="horizontal"
      aria-label="Resize the panel"
      aria-valuenow={height}
      tabindex="0"
      onpointerdown={drag}
      onkeydown={nudge}
    ></div>
  {/if}
  <div class="strip" role="tablist">
    <button role="tab" aria-selected={open && tab === "variables"} class:on={open && tab === "variables"} onclick={() => pick("variables")}>
      Variables <span class="n">{variables.length}</span>
    </button>
    <button role="tab" aria-selected={open && tab === "proposals"} class:on={open && tab === "proposals"} onclick={() => pick("proposals")}>
      Proposals {#if openProposals}<span class="n due">{openProposals}</span>{/if}
    </button>
    <span class="spacer"></span>
    <button class="chev" title="{open ? 'Hide' : 'Show'} the panel (⌘J)" aria-label="{open ? 'Hide' : 'Show'} the panel" onclick={() => (open = !open)}>
      {open ? "▾" : "▴"}
    </button>
  </div>
  {#if open}
    <div class="body">
      {#if tab === "variables"}
        <VariableTable {variables} {editable} onjump={onjumpcell} {ondescribe} />
      {:else}
        <div class="proposals">
          <ProposalsPanel {proposals} {editable} {onaccept} {onreject} {onjumpcell} />
        </div>
      {/if}
    </div>
  {/if}
</section>

<style>
  .bottom {
    position: relative; display: grid; grid-template-rows: auto; border-top: 1px solid var(--border);
    background: var(--panel); min-height: 0;
  }
  .bottom.open { grid-template-rows: auto 1fr; }
  .grip { position: absolute; left: 0; right: 0; top: -3px; height: 6px; cursor: ns-resize; z-index: 2; }
  .grip:hover, .grip:focus-visible { background: color-mix(in srgb, var(--accent) 35%, transparent); outline: none; }
  .strip { display: flex; align-items: center; gap: 2px; padding: 0 8px; border-bottom: 1px solid transparent; }
  .open .strip { border-bottom-color: var(--border); }
  .strip button {
    border: none; background: none; border-radius: 0; padding: 5px 10px 4px; font-size: 12.5px; color: var(--muted);
    border-bottom: 2px solid transparent; margin-bottom: -1px;
  }
  .strip button:hover { color: var(--text); }
  .strip button.on { color: var(--text); border-bottom-color: var(--accent); font-weight: 600; }
  .n {
    font-size: 10.5px; padding: 0 5px; border-radius: 7px; background: var(--sunken); border: 1px solid var(--border);
    font-weight: normal; margin-left: 3px;
  }
  .n.due { background: var(--accent); border-color: var(--accent); color: #fff; }
  .spacer { flex: 1; }
  .chev { font-size: 12px !important; }
  .body { min-height: 0; overflow: hidden; }
  .proposals { height: 100%; overflow: auto; }
  .proposals > :global(*) { max-width: 980px; }
</style>
