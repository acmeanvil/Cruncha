<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- A snippet's title, summary, category and tags (ADR-0014 §5): Save as snippet, Edit details. -->
<script lang="ts">
  import { untrack } from "svelte";
  import Modal from "./Modal.svelte";
  import type { SnippetDetails } from "./api";
  import { splitList } from "./templateRef";

  let {
    heading,
    action,
    initial,
    note = null,
    error = null,
    onsave,
    oncancel,
  }: {
    heading: string;
    action: string;
    initial: SnippetDetails;
    /** What will be saved ("3 cells; inputs D, L"). */
    note?: string | null;
    error?: string | null;
    onsave: (d: SnippetDetails) => void;
    oncancel: () => void;
  } = $props();

  let title = $state(untrack(() => initial.title));
  let summary = $state(untrack(() => initial.summary ?? ""));
  let category = $state(untrack(() => initial.category.join(" / ")));
  let tags = $state(untrack(() => initial.tags.join(", ")));
  let titleEl: HTMLInputElement | undefined = $state();
  $effect(() => titleEl?.focus());

  function save() {
    if (!title.trim()) return;
    onsave({ title: title.trim(), summary: summary.trim() || null, category: splitList(category, "/"), tags: splitList(tags, ",") });
  }
</script>

<Modal title={heading} onclose={oncancel} width={480}>
  <form class="form" onsubmit={(e) => (e.preventDefault(), save())}>
    {#if note}<p class="small muted">{note}</p>{/if}
    <label>Title <input bind:this={titleEl} bind:value={title} placeholder="Load combination, LRFD" /></label>
    <label>Summary <textarea rows="2" bind:value={summary} placeholder="What it calculates, in a sentence"></textarea></label>
    <label>Category <input bind:value={category} placeholder="Structural engineering / Loads" /></label>
    <label>Tags <input bind:value={tags} placeholder="ASCE 7, LRFD" /></label>
    {#if error}<p class="small err">{error}</p>{/if}
  </form>
  {#snippet footer()}
    <button onclick={oncancel}>Cancel</button>
    <button class="primary" disabled={!title.trim()} onclick={save}>{action}</button>
  {/snippet}
</Modal>

<style>
  .form { display: grid; gap: 10px; }
  label { display: grid; gap: 4px; font-size: 12.5px; color: var(--muted); }
  input, textarea { font: inherit; font-size: 13.5px; color: var(--text); }
  textarea { resize: vertical; }
  .err { color: var(--danger); }
</style>
