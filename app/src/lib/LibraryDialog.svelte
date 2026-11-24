<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The Library page (ADR-0014 §4, §5): where the user's library is, import, the trash, and
     built-in templates the user hid. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import Modal from "./Modal.svelte";
  import { api, type LibraryInfo, type TemplateSummary } from "./api";

  let {
    templates,
    onchanged,
    onclose,
  }: {
    templates: TemplateSummary[];
    /** The libraries changed (reload the list); `key`: select that template. */
    onchanged: (key?: string) => void;
    onclose: () => void;
  } = $props();

  let info = $state<LibraryInfo | null>(null);
  let trash = $state<{ name: string; title: string; id: string | null }[]>([]);
  let error = $state<string | null>(null);
  let note = $state<string | null>(null);
  const hidden = $derived(templates.filter((t) => t.hidden));

  async function load() {
    try {
      info = await api.libraryInfo();
      trash = await api.templateTrash();
    } catch (e) {
      error = String(e);
    }
  }
  onMount(load);

  async function act(f: () => Promise<unknown>, done?: string) {
    error = null;
    note = null;
    try {
      await f();
      if (done) note = done;
      await load();
      onchanged();
    } catch (e) {
      error = String(e);
    }
  }

  async function chooseFolder() {
    const dir = await openDialog({ directory: true, title: "Folder for your templates and snippets", defaultPath: info?.dir });
    if (typeof dir === "string") await act(() => api.setLibraryDir(dir));
  }

  async function importFiles() {
    const files = await openDialog({ multiple: true, title: "Import templates", filters: [{ name: "Cruncha template", extensions: ["md"] }] });
    const paths = files == null ? [] : Array.isArray(files) ? files : [files];
    if (paths.length === 0) return;
    await act(async () => {
      const keys = await api.templateImport(paths);
      onchanged(keys[0]);
    }, `Imported ${paths.length} ${paths.length === 1 ? "file" : "files"}.`);
  }
</script>

<Modal title="Template library" {onclose} width={600}>
  <section>
    <h3>Your library</h3>
    <p class="small muted">Your snippets and templates are <code>.cruncha.md</code> files in this folder. Files you edit or add outside Cruncha appear the next time the list loads. Point it at a shared drive or a git repository to share a library with a team.</p>
    {#if info}
      <div class="row">
        <code class="path" title={info.dir}>{info.dir}</code>
        <button onclick={() => act(() => api.libraryReveal())}>Show in Finder</button>
      </div>
      <div class="row">
        <button onclick={chooseFolder}>Change folder…</button>
        {#if info.custom}<button onclick={() => act(() => api.setLibraryDir(null))}>Use the default folder</button>{/if}
        <span class="spacer"></span>
        <button onclick={importFiles}>Import…</button>
      </div>
    {/if}
  </section>

  <section>
    <h3>Trash</h3>
    {#if trash.length === 0}
      <p class="small muted">Deleted snippets go here until you empty the trash.</p>
    {:else}
      <ul>
        {#each trash as x (x.name)}
          <li>
            <span>{x.title}</span>
            <span class="spacer"></span>
            <button class="small" onclick={() => act(async () => onchanged(await api.templateRestore(x.name)), `Restored “${x.title}”.`)}>Restore</button>
          </li>
        {/each}
      </ul>
      <button class="danger" onclick={() => act(() => api.templateEmptyTrash(), "The trash is empty.")}>Empty trash ({trash.length})</button>
    {/if}
  </section>

  <section>
    <h3>Hidden built-in templates</h3>
    {#if hidden.length === 0}
      <p class="small muted">None. Built-in templates can't be deleted, only hidden from the list.</p>
    {:else}
      <ul>
        {#each hidden as t (t.key)}
          <li>
            <span>{t.title}</span>
            <span class="spacer"></span>
            <button class="small" onclick={() => act(() => api.templateHide(t.id, false))}>Show</button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
  {#if note}<p class="small ok">{note}</p>{/if}
  {#if error}<p class="small err">{error}</p>{/if}
  {#snippet footer()}
    <button class="primary" onclick={onclose}>Done</button>
  {/snippet}
</Modal>

<style>
  section { margin-bottom: 16px; }
  h3 { font-size: 13px; margin: 0 0 4px; }
  .row { display: flex; gap: 6px; align-items: center; margin-top: 6px; }
  .path { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font: 12px var(--mono); background: var(--sunken); padding: 4px 6px; border-radius: 4px; }
  ul { list-style: none; margin: 4px 0 8px; padding: 0; }
  li { display: flex; align-items: center; gap: 6px; padding: 4px 0; border-bottom: 1px solid var(--border); font-size: 13px; }
  li button { padding: 1px 8px; font-size: 12px; }
  .spacer { flex: 1; }
  .err { color: var(--danger); }
  .ok { color: var(--ok, var(--accent)); }
</style>
