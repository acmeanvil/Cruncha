<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Agents: let AI agents on this computer (Claude Code, Codex, Gemini CLI, …) use the open
     document through Cruncha's MCP server (ADR-0009). Off until the user turns it on; the
     server listens on 127.0.0.1 only, with a token that changes every time it starts. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { api, type AgentServerView } from "./api";
  import Modal from "./Modal.svelte";

  let { onclose, onchange }: { onclose: () => void; onchange: (running: boolean) => void } = $props();

  let server = $state<AgentServerView | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let copied = $state<string | null>(null);
  let showToken = $state(false);

  onMount(async () => {
    try {
      server = await api.agentServerStatus();
    } catch (e) {
      error = String(e);
    }
  });

  async function toggle() {
    busy = true;
    error = null;
    try {
      if (server) {
        await api.agentServerStop();
        server = null;
      } else {
        server = await api.agentServerStart();
      }
      onchange(server != null);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function copy(what: string, text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copied = what;
      setTimeout(() => (copied = copied === what ? null : copied), 1500);
    } catch (e) {
      error = `Couldn't copy: ${e}`;
    }
  }

  const masked = (s: string) => (showToken ? s : s.replace(server?.token ?? "\u0000", "•".repeat(12)));
</script>

<Modal title="Agents" {onclose}>
  <p>
    AI agents on this computer can read this document through Cruncha: its cells, the engine's results,
    checks and history. They can't change it yet; proposed edits come next.
  </p>
  <label class="switch">
    <input type="checkbox" checked={server != null} disabled={busy} onchange={toggle} />
    Allow agents on this computer to use the open document
  </label>
  {#if server}
    <h3>Claude Code</h3>
    <p class="small muted">Run this once in a terminal, then ask Claude about the document.</p>
    <div class="cmd">
      <code>{masked(server.claudeCommand)}</code>
      <button onclick={() => copy("claude", server!.claudeCommand)}>{copied === "claude" ? "Copied" : "Copy"}</button>
    </div>
    <h3>Other agents</h3>
    <p class="small muted">An MCP server over HTTP. Add this to the agent's MCP settings:</p>
    <div class="cmd">
      <pre>{masked(server.configJson)}</pre>
      <button onclick={() => copy("json", server!.configJson)}>{copied === "json" ? "Copied" : "Copy"}</button>
    </div>
    <p class="small muted">
      <button class="link" onclick={() => (showToken = !showToken)}>{showToken ? "Hide" : "Show"} the token</button>
      · Only programs on this computer that have the token can connect. Turning this off, or quitting
      Cruncha, disconnects them; the token changes every time you turn it on.
    </p>
  {/if}
  {#if error}<p class="error">{error}</p>{/if}
  {#snippet footer()}
    <button onclick={onclose}>Close</button>
  {/snippet}
</Modal>

<style>
  .switch { display: flex; gap: 8px; align-items: center; margin: 10px 0; font-weight: 600; }
  h3 { margin: 14px 0 2px; font-size: 14px; }
  .cmd { display: flex; gap: 8px; align-items: flex-start; }
  .cmd code, .cmd pre {
    flex: 1; min-width: 0; margin: 0; padding: 6px 8px; border-radius: 4px; background: var(--sunken);
    font: 12px/1.45 var(--mono); white-space: pre-wrap; word-break: break-all;
  }
  .error { color: var(--danger); }
</style>
