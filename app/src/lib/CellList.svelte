<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Read-only rendering of a document's cells. The editor (ProseMirror, Temml, MathLive)
     replaces this; for now calc cells show their source. -->
<script lang="ts">
  import type { Cell } from "./api";
  import { proseText } from "./format";

  let { cells }: { cells: Cell[] } = $props();

  const code = new Set(["python", "rust", "tex"]);
</script>

{#if cells.length === 0}
  <p class="empty muted">This version has no cells.</p>
{/if}
{#each cells as c (c.id)}
  <div class="cell {c.kind}">
    <span class="kind small muted">{c.kind}</span>
    {#if c.kind === "prose"}
      <p>{proseText(c.source)}</p>
    {:else if code.has(c.kind)}
      <pre>{c.source}</pre>
    {:else}
      <code>{c.source}</code>
    {/if}
    {#if c.provenance.state === "ai_proposed"}
      <span class="ai small">AI proposal</span>
    {:else if c.provenance.state === "ai_accepted"}
      <span class="ai small">AI, accepted by {c.provenance.by}</span>
    {/if}
  </div>
{/each}

<style>
  .cell {
    position: relative; padding: 6px 12px 6px 84px; border-radius: 6px; margin: 2px 0;
  }
  .cell:hover { background: var(--sunken); }
  .kind { position: absolute; left: 12px; top: 9px; width: 64px; text-align: right; }
  p { margin: 2px 0; font-size: 15px; }
  code { font-family: var(--mono); font-size: 14px; color: var(--code); }
  pre {
    font-family: var(--mono); font-size: 13px; margin: 0; padding: 8px 10px;
    background: var(--sunken); border-radius: 6px; overflow: auto;
  }
  .ai { margin-left: 10px; color: var(--accent); }
  .empty { padding: 24px; }
</style>
