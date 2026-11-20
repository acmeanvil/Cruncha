<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    onclose,
    children,
    footer,
  }: { title: string; onclose: () => void; children: Snippet; footer?: Snippet } = $props();
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />

<div class="backdrop" role="presentation" onclick={(e) => e.target === e.currentTarget && onclose()}>
  <div class="modal" role="dialog" aria-modal="true" aria-label={title}>
    <header>
      <h2>{title}</h2>
      <button class="close" aria-label="Close" onclick={onclose}>×</button>
    </header>
    <div class="body">{@render children()}</div>
    {#if footer}<footer>{@render footer()}</footer>{/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed; inset: 0; background: rgb(0 0 0 / 0.35);
    display: grid; place-items: center; z-index: 10; padding: 16px;
  }
  .modal {
    background: var(--panel); border: 1px solid var(--border); border-radius: 10px;
    width: min(560px, 100%); max-height: calc(100vh - 32px); overflow: auto;
    box-shadow: 0 12px 40px rgb(0 0 0 / 0.25);
  }
  header { display: flex; align-items: center; justify-content: space-between; padding: 14px 18px 0; }
  h2 { margin: 0; font-size: 16px; }
  .close { border: none; font-size: 20px; line-height: 1; padding: 2px 6px; background: none; }
  .body { padding: 12px 18px; }
  footer {
    display: flex; justify-content: flex-end; gap: 8px; padding: 12px 18px;
    border-top: 1px solid var(--border);
  }
</style>
