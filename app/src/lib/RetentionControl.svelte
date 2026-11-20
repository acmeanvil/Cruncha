<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The history-retention control (ADR-0011 §5), used on the Preferences page and in
     document settings. -->
<script lang="ts">
  import { untrack } from "svelte";
  import type { Retention } from "./api";
  import { isKeepEverything } from "./format";

  let { policy = $bindable(), disabled = false }: { policy: Retention; disabled?: boolean } =
    $props();

  const init = untrack(() => policy);
  const floor = init.floor ?? 100;
  let limited = $state(!isKeepEverything(init));
  let savesOn = $state(init.saves != null);
  let saves = $state(init.saves ?? 1000);
  let sizeOn = $state(init.size_bytes != null);
  let sizeMb = $state(init.size_bytes != null ? Math.max(1, Math.round(init.size_bytes / 1e6)) : 500);
  let daysOn = $state(init.activity_days != null);
  let days = $state(init.activity_days ?? 90);

  const whole = (v: number) => Math.max(1, Math.floor(Number(v) || 1));

  $effect(() => {
    policy = {
      saves: limited && savesOn ? whole(saves) : null,
      size_bytes: limited && sizeOn ? whole(sizeMb) * 1_000_000 : null,
      activity_days: limited && daysOn ? whole(days) : null,
      floor,
    };
  });

  function chooseLimited() {
    if (!savesOn && !sizeOn && !daysOn) savesOn = true;
  }
</script>

<fieldset {disabled}>
  <label class="choice">
    <input type="radio" bind:group={limited} value={false} />
    Keep everything
  </label>
  <label class="choice">
    <input type="radio" bind:group={limited} value={true} onchange={chooseLimited} />
    Limit history to
  </label>
  <div class="limits" class:off={!limited}>
    <label>
      <input type="checkbox" bind:checked={savesOn} disabled={!limited} />
      the last <input type="number" min="1" bind:value={saves} disabled={!limited || !savesOn} /> saves
    </label>
    <label>
      <input type="checkbox" bind:checked={sizeOn} disabled={!limited} />
      up to <input type="number" min="1" bind:value={sizeMb} disabled={!limited || !sizeOn} /> MB on disk
    </label>
    <label>
      <input type="checkbox" bind:checked={daysOn} disabled={!limited} />
      <input type="number" min="1" bind:value={days} disabled={!limited || !daysOn} /> days of activity
    </label>
    {#if limited && !savesOn && !sizeOn && !daysOn}
      <p class="small muted">No limit is ticked, so everything is kept.</p>
    {/if}
  </div>
  <p class="help small muted">
    Time limits count back from your last edit; documents you aren't editing are never trimmed.
    Named versions are always kept, and so are the newest {floor} versions. When several limits
    are ticked, the strictest wins.
  </p>
</fieldset>

<style>
  fieldset { border: none; padding: 0; margin: 0; }
  .choice { display: block; margin: 4px 0; font-weight: 500; }
  .limits { margin: 4px 0 0 24px; display: grid; gap: 6px; }
  .limits.off { opacity: 0.55; }
  .help { margin: 10px 0 0; }
</style>
