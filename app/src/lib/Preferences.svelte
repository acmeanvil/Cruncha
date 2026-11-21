<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Preferences: editor text sizes, the office's report style, and the app-wide default
     retention for documents (ADR-0011). -->
<script lang="ts">
  import { untrack } from "svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import {
    api,
    DEFAULT_REPORT_STYLE,
    DEFAULT_TEXT_SIZES,
    type DocInfo,
    type Prefs,
    type Preview,
    type ReportStyle,
    type TextSizes,
  } from "./api";
  import Modal from "./Modal.svelte";
  import PruneConfirm from "./PruneConfirm.svelte";
  import RetentionControl from "./RetentionControl.svelte";
  import { isKeepEverything } from "./format";

  let {
    prefs,
    doc,
    onclose,
    onsaved,
  }: {
    prefs: Prefs;
    doc: DocInfo | null;
    onclose: () => void;
    onsaved: (p: Prefs) => void;
  } = $props();

  let policy = $state($state.snapshot(untrack(() => prefs.defaultRetention)));
  let sizes = $state<TextSizes>({ ...DEFAULT_TEXT_SIZES, ...untrack(() => prefs.textSizes) });
  const SIZE_ROWS: { key: keyof TextSizes; label: string; hint: string }[] = [
    { key: "text", label: "Text", hint: "prose; headings scale with it (H1 2×, H2 1.5×, H3 1.17×)" },
    { key: "calc", label: "Calcs", hint: "typeset math, sources and descriptions" },
    { key: "code", label: "Code", hint: "Python cells and their output" },
  ];
  const clamp = (n: number, d: number) => (Number.isFinite(n) ? Math.min(32, Math.max(9, n)) : d);
  let style = $state<ReportStyle>({ ...DEFAULT_REPORT_STYLE, ...untrack(() => prefs.reportStyle) });

  async function chooseLogo() {
    const path = await openDialog({
      title: "Logo for report headers",
      filters: [{ name: "Image", extensions: ["png", "jpg", "jpeg", "svg"] }],
    });
    if (typeof path === "string") style.logoPath = path;
  }

  let confirm = $state<Preview | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);

  async function save() {
    busy = true;
    error = null;
    try {
      // The open document follows the default, so a stricter default may trim it.
      if (!confirm && doc && doc.retention == null && !isKeepEverything(policy)) {
        const p = await api.retentionPreview(policy);
        if (p.cutoff != null) {
          confirm = p;
          return;
        }
      }
      const textSizes = {
        text: clamp(sizes.text, DEFAULT_TEXT_SIZES.text),
        calc: clamp(sizes.calc, DEFAULT_TEXT_SIZES.calc),
        code: clamp(sizes.code, DEFAULT_TEXT_SIZES.code),
      };
      const reportStyle = {
        ...$state.snapshot(style),
        accent: /^#[0-9a-f]{6}$/i.test(style.accent) ? style.accent : DEFAULT_REPORT_STYLE.accent,
      };
      const next = { ...$state.snapshot(prefs), defaultRetention: $state.snapshot(policy), textSizes, reportStyle };
      await api.setPrefs(next);
      onsaved(next);
      onclose();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<Modal title="Preferences" {onclose}>
  <h3>Text sizes</h3>
  <div class="sizes">
    {#each SIZE_ROWS as row (row.key)}
      <label for="size-{row.key}">{row.label}</label>
      <span><input id="size-{row.key}" type="number" min="9" max="32" step="0.5" bind:value={sizes[row.key]} /> px</span>
      <span class="small muted">{row.hint}</span>
    {/each}
  </div>
  <p class="small"><button class="link" onclick={() => (sizes = { ...DEFAULT_TEXT_SIZES })}>Reset to defaults</button></p>
  <h3>Reports</h3>
  <div class="sizes">
    <label for="rs-company">Company</label>
    <input id="rs-company" class="wide" bind:value={style.company} placeholder="Shown in the page header" />
    <span></span>
    <label for="rs-logo">Logo</label>
    <span class="logo">
      <button id="rs-logo" onclick={chooseLogo}>{style.logoPath ? "Change…" : "Choose…"}</button>
      {#if style.logoPath}
        <span class="small muted path" title={style.logoPath}>{style.logoPath.split(/[\\/]/).pop()}</span>
        <button class="link" onclick={() => (style.logoPath = null)}>Remove</button>
      {/if}
    </span>
    <span class="small muted">replaces the company name</span>
    <label for="rs-accent">Accent</label>
    <span><input id="rs-accent" type="color" bind:value={style.accent} /></span>
    <span class="small muted">headings and rules</span>
    <label for="rs-paper">Paper</label>
    <span>
      <select id="rs-paper" bind:value={style.paper}>
        <option value="us-letter">US Letter</option>
        <option value="a4">A4</option>
      </select>
    </span>
    <span></span>
  </div>
  <h3>History</h3>
  <p class="small muted">
    Every change is saved as a version. This is the default for documents that don't set their
    own policy in Document settings.
  </p>
  <RetentionControl bind:policy disabled={confirm != null} />
  {#if confirm && doc}
    <PruneConfirm preview={confirm} docName={doc.name} />
  {/if}
  {#if error}<p class="error">{error}</p>{/if}
  {#snippet footer()}
    {#if confirm}
      <button onclick={() => (confirm = null)}>Back</button>
      <button class="danger" disabled={busy} onclick={save}>Save</button>
    {:else}
      <button onclick={onclose}>Cancel</button>
      <button class="primary" disabled={busy} onclick={save}>Save</button>
    {/if}
  {/snippet}
</Modal>

<style>
  h3 { margin: 4px 0; font-size: 14px; }
  h3 + .sizes { margin-top: 8px; }
  .sizes { display: grid; grid-template-columns: auto auto 1fr; gap: 6px 12px; align-items: center; }
  .sizes input { width: 5em; }
  .sizes input.wide { width: 16em; }
  .sizes input[type="color"] { width: 3em; height: 1.8em; padding: 0 2px; }
  .logo { display: flex; gap: 8px; align-items: center; min-width: 0; }
  .path { max-width: 12em; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  p + h3 { margin-top: 14px; }
  .error { color: var(--danger); }
</style>
