<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Editor type sizes (text, calcs, code), used by Settings (the default) and Document settings. -->
<script lang="ts">
  import { DEFAULT_TEXT_SIZES, type TextSizes } from "./api";

  let {
    sizes,
    disabled = false,
    onchange,
  }: { sizes: TextSizes; disabled?: boolean; onchange: (t: TextSizes) => void } = $props();

  const ROWS: { key: keyof TextSizes; label: string; hint: string }[] = [
    { key: "text", label: "Text", hint: "prose; headings scale with it (H1 2×, H2 1.5×, H3 1.17×)" },
    { key: "calc", label: "Calcs", hint: "typeset math, sources and descriptions" },
    { key: "code", label: "Code", hint: "Python cells and their output" },
  ];
  const clamp = (n: number, d: number) => (Number.isFinite(n) ? Math.min(32, Math.max(9, n)) : d);

  function set(key: keyof TextSizes, value: number) {
    onchange({ ...sizes, [key]: clamp(value, DEFAULT_TEXT_SIZES[key]) });
  }
</script>

<div class="sizes">
  {#each ROWS as row (row.key)}
    <label for="size-{row.key}">{row.label}</label>
    <span>
      <input
        id="size-{row.key}"
        type="number"
        min="9"
        max="32"
        step="0.5"
        value={sizes[row.key]}
        {disabled}
        onchange={(e) => set(row.key, Number((e.currentTarget as HTMLInputElement).value))}
      /> px
    </span>
    <span class="small muted">{row.hint}</span>
  {/each}
</div>
{#if !disabled}
  <p class="small"><button class="link" onclick={() => onchange({ ...DEFAULT_TEXT_SIZES })}>Reset to defaults</button></p>
{/if}

<style>
  .sizes { display: grid; grid-template-columns: auto auto 1fr; gap: 6px 12px; align-items: center; }
  .sizes input { width: 5em; }
  p { margin: 6px 0 0; }
</style>
