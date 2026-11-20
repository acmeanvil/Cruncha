<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Confirmation before a stricter policy takes effect, backed by retention_preview. -->
<script lang="ts">
  import type { Preview } from "./api";
  import { bytes, date } from "./format";

  let { preview, docName }: { preview: Preview; docName: string } = $props();
</script>

<div class="confirm" role="alert">
  <p><strong>This will permanently remove history from “{docName}”.</strong></p>
  <ul>
    <li>
      Versions before <strong>{preview.cutoff}</strong>{#if preview.cutoffTimestampMs != null}
        ({date(preview.cutoffTimestampMs)}){/if} will become unavailable.
    </li>
    <li>About <strong>{bytes(preview.bytesFreed)}</strong> will be freed.</li>
    {#if preview.refsToPin.length}
      <li>Named versions kept as full copies: {preview.refsToPin.join(", ")}.</li>
    {/if}
    {#if preview.waitsForEdit}
      <li>Nothing is removed until the document is next edited.</li>
    {/if}
  </ul>
  <p class="warn">This cannot be undone.</p>
</div>

<style>
  .confirm {
    background: var(--danger-bg); border: 1px solid var(--danger); border-radius: 8px;
    padding: 4px 14px; margin-top: 12px;
  }
  ul { padding-left: 18px; margin: 6px 0; }
  .warn { color: var(--danger); font-weight: 600; }
</style>
