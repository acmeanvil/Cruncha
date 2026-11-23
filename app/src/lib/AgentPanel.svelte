<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- The agent panel (ADR-0009): an ACP agent (Claude Code, Codex, Gemini CLI, …) working on the
     open document through Cruncha's MCP tools. Modelled on looking_glass's session view:
     streaming replies, collapsible thinking, tool cards, a plan, permission prompts with
     Allow / Always / Reject, the agent's own model and mode menus, and token usage. -->
<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount, tick } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import {
    api,
    NO_AGENT_PREF,
    NO_AGENT_SETTINGS,
    type AgentDef,
    type ConfigOption,
    type KnownModel,
    type AgentEntry,
    type ConversationSummary,
    type PanelView,
    type PermissionView,
  } from "./api";
  import { linkCells, renderMarkdown } from "./agentText";
  import { date } from "./format";

  let {
    cellIds,
    onjumpcell,
    onclose,
    onsettings,
    docId = null,
    docName,
  }: {
    /** Cell IDs of the open document (mentions become links). */
    cellIds: Set<string>;
    onjumpcell: (id: string) => void;
    onclose: () => void;
    /** Open Settings at a section (`agents`, `external`). */
    onsettings: (section: "agents" | "external") => void;
    /** The active tab's document. */
    docId?: string | null;
    /** A document's name, for the note when the conversation is in another tab. */
    docName?: (id: string) => string | null;
  } = $props();

  /** Models each agent offered (from Settings), and the one to start with. */
  let known = $state<Record<string, KnownModel[]>>({});
  let startModel = $state("");
  let loadingModels = $state(false);

  async function refreshKnown() {
    const prefs = await api.getPrefs().catch(() => null);
    const agents = prefs?.agents ?? NO_AGENT_SETTINGS;
    known = Object.fromEntries(Object.entries(agents.agents).map(([id, p]) => [id, p.knownModels ?? []]));
    startModel = agents.agents[choice]?.model ?? "";
  }

  async function loadModels() {
    loadingModels = true;
    await act(async () => {
      await api.agentModels(choice);
      await refreshKnown();
    });
    loadingModels = false;
  }

  /** Remember the model picked for this agent (the same setting as in Settings). */
  async function pickModel(model: string) {
    startModel = model;
    await act(async () => {
      const prefs = await api.getPrefs();
      const agents = prefs.agents ?? NO_AGENT_SETTINGS;
      const pref = { ...NO_AGENT_PREF, ...agents.agents[choice], model: model || null };
      await api.setPrefs({ ...prefs, agents: { ...agents, agents: { ...agents.agents, [choice]: pref } } });
    });
  }

  /** The model to select once the agent that just started offers its menus (Settings → Agents). */
  let pendingModel: { agent: string; model: string } | null = null;

  /** Apply the saved model, and remember the models the agent offers (for Settings). */
  async function useModelPrefs(v: PanelView) {
    const model = v.options.find((o: ConfigOption) => o.category === "Model");
    if (!v.ready || !v.agentId || !model) return;
    if (pendingModel?.agent === v.agentId) {
      const want = pendingModel.model;
      pendingModel = null;
      if (want !== model.current && model.choices.some((c) => c.value === want)) {
        await act(() => api.agentSetOption(model.id, want));
      }
    }
    try {
      const prefs = await api.getPrefs();
      const agents = prefs.agents ?? NO_AGENT_SETTINGS;
      const pref = agents.agents[v.agentId] ?? NO_AGENT_PREF;
      const known = model.choices.map((c) => ({ value: c.value, name: c.name, group: c.group }));
      if (JSON.stringify(known) === JSON.stringify(pref.knownModels)) return;
      await api.setPrefs({
        ...prefs,
        agents: { ...agents, agents: { ...agents.agents, [v.agentId]: { ...pref, knownModels: known } } },
      });
    } catch {
      // Remembering the menu is a convenience; the session works without it.
    }
  }

  let view = $state<PanelView | null>(null);
  let catalog = $state<AgentDef[]>([]);
  let choice = $state("claude");
  const choiceModels = $derived(known[choice] ?? []);
  let prompt = $state("");
  let error = $state<string | null>(null);
  let scroller = $state<HTMLElement | null>(null);
  let pinned = true;
  let past = $state<ConversationSummary[] | null>(null);
  let reading = $state<{ summary: ConversationSummary; entries: AgentEntry[] } | null>(null);
  const open = new SvelteSet<string>();

  const entries = $derived(reading ? reading.entries : (view?.entries ?? []));
  const running = $derived(view?.running ?? false);

  onMount(() => {
    let off: (() => void) | undefined;
    void (async () => {
      try {
        catalog = await api.agentCatalog();
        view = await api.agentPanelView();
        if (view.agentId) choice = view.agentId;
        else {
          const preferred = (await api.getPrefs()).agents?.defaultAgent;
          let saved: string | null = null;
          try {
            saved = localStorage.getItem("cruncha.agent");
          } catch {}
          const pick = [preferred, saved].find((id) => id && catalog.some((a) => a.id === id));
          if (pick) choice = pick;
        }
        await refreshKnown();
      } catch (e) {
        error = String(e);
      }
      off = await listen<PanelView>("agent-panel", (e) => {
        view = e.payload;
        void follow();
        void useModelPrefs(e.payload);
      });
      await follow();
    })();
    return () => off?.();
  });

  async function follow() {
    await tick();
    if (pinned && scroller) scroller.scrollTop = scroller.scrollHeight;
  }

  function onscroll() {
    if (!scroller) return;
    pinned = scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight < 40;
  }

  async function act<T>(f: () => Promise<T>): Promise<T | undefined> {
    error = null;
    try {
      return await f();
    } catch (e) {
      error = String(e);
    }
  }

  async function start() {
    try {
      localStorage.setItem("cruncha.agent", choice);
    } catch {}
    reading = null;
    // The built-in agent gets its model at launch; adapters get it from their menu.
    const model = (await api.getPrefs().catch(() => null))?.agents?.agents[choice]?.model;
    pendingModel = model && choice !== "looking-glass" ? { agent: choice, model } : null;
    const v = await act(() => api.agentStart(choice));
    if (v) {
      view = v;
      void useModelPrefs(v);
    }
  }

  async function send() {
    const text = prompt.trim();
    if (!text || !view?.ready || view.busy) return;
    prompt = "";
    pinned = true;
    const v = await act(() => api.agentPrompt(text));
    if (v) view = v;
    await follow();
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey && !e.isComposing) {
      e.preventDefault();
      void send();
    } else if (e.key === "Escape" && view?.busy) {
      e.preventDefault();
      void act(() => api.agentCancel()).then((v) => v && (view = v));
    }
  }

  async function answer(p: PermissionView, option: string | null) {
    const v = await act(() => api.agentPermission(p.id, option));
    if (v) view = v;
  }

  async function showPast() {
    past = (await act(() => api.agentConversations())) ?? [];
  }

  async function openPast(c: ConversationSummary) {
    const r = await act(() => api.agentConversation(c.id));
    if (r) {
      reading = r;
      past = null;
    }
  }

  /** A tool acts outside the document (files or commands on this computer). */
  const outside = (kind: string) => ["edit", "delete", "move", "execute"].includes(kind);
  const toolOf = (p: PermissionView) =>
    view?.entries.find((e) => e.type === "tool" && e.id === p.toolCallId) as Extract<AgentEntry, { type: "tool" }> | undefined;

  /** Render Markdown and link cell IDs. */
  function rich(node: HTMLElement, text: string) {
    const draw = (t: string) => {
      node.innerHTML = renderMarkdown(t);
      linkCells(node, cellIds);
    };
    draw(text);
    return { update: draw };
  }

  function onclick(e: MouseEvent) {
    const t = e.target as HTMLElement;
    const ref = t.closest<HTMLElement>("button.cellref");
    if (ref?.dataset.cell) {
      onjumpcell(ref.dataset.cell);
      return;
    }
    const a = t.closest<HTMLAnchorElement>("a[href]");
    if (a) {
      e.preventDefault();
      if (/^https?:/.test(a.href)) void act(() => api.openPath(a.href));
    }
  }

  const usage = $derived.by(() => {
    if (!view || (!view.used && !view.totals)) return null;
    const pct = view.size ? Math.round((100 * view.used) / view.size) : null;
    const k = (n: number) => (n >= 1000 ? `${(n / 1000).toFixed(n >= 10000 ? 0 : 1)}k` : `${n}`);
    return {
      label: pct != null ? `${pct}% context` : `${k(view.used)} tokens`,
      title: view.totals
        ? `Input ${view.totals.input_tokens}, output ${view.totals.output_tokens}, cache read ${view.totals.cache_read_tokens}, requests ${view.totals.requests}`
        : `${view.used} of ${view.size || "?"} tokens in context`,
    };
  });

  const toolIcon: Record<string, string> = {
    read: "◧", search: "⌕", edit: "✎", delete: "✕", move: "⇄", execute: "▶", think: "…", fetch: "⇣", other: "◇",
  };
  const shortTitle = (t: string) => t.replace(/^mcp__cruncha__/, "").replace(/^cruncha[._:]/, "");
</script>

<!-- Clicks on cell links and links inside replies are handled here (event delegation). -->
<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
<aside class="panel" {onclick}>
  {#if view?.running && view.docId && docId && view.docId !== docId}
    <p class="elsewhere small">
      This conversation belongs to <strong>{docName?.(view.docId) ?? "another tab"}</strong>; the agent can still read and propose
      to this document when you name it.
    </p>
  {/if}
  <header>
    <strong>Agent</strong>
    {#if view?.agentId && !reading}
      <span class="who" title={view.agentVersion ?? ""}>{view.agentName}</span>
      {#each view.options as o (o.id)}
        <select
          class="opt"
          title={o.name}
          value={o.current}
          disabled={!view.ready}
          onchange={(e) => void act(() => api.agentSetOption(o.id, (e.currentTarget as HTMLSelectElement).value))}
        >
          {#each o.choices as c (c.value)}
            <option value={c.value} title={c.description ?? ""}>{c.group ? `${c.group} · ` : ""}{c.name}</option>
          {/each}
        </select>
      {/each}
      {#if usage}<span class="usage" title={usage.title}>{usage.label}</span>{/if}
    {/if}
    <span class="spacer"></span>
    <button class="icon" title="Earlier conversations" aria-label="Earlier conversations" onclick={showPast}>🕘</button>
    <button class="icon" title="Close the panel" aria-label="Close the panel" onclick={onclose}>×</button>
  </header>

  {#if past}
    <div class="past">
      <div class="past-head"><strong>Earlier conversations</strong><button class="link" onclick={() => (past = null)}>Back</button></div>
      {#if past.length === 0}<p class="small muted">None yet for this document.</p>{/if}
      {#each past as c (c.id)}
        <button class="conv" onclick={() => openPast(c)}>
          <span>{c.title || "(no prompt)"}</span>
          <span class="small muted">{c.agent} · {date(c.updatedMs)} · {c.turns} {c.turns === 1 ? "turn" : "turns"} · from version {c.version}</span>
        </button>
      {/each}
    </div>
  {:else}
    {#if reading}
      <div class="reading small">
        Reading an earlier conversation with {reading.summary.agent} ({date(reading.summary.updatedMs)}).
        <button class="link" onclick={() => (reading = null)}>Back to the current one</button>
      </div>
    {/if}
    {#if view?.plan.length && !reading}
      <ol class="plan">
        {#each view.plan as item, i (i)}
          <li class={item.status.toLowerCase()}>{item.content}</li>
        {/each}
      </ol>
    {/if}
    <div class="log" bind:this={scroller} {onscroll}>
      {#if entries.length === 0}
        <div class="empty">
          <p><strong>Ask an agent about this document.</strong></p>
          <p class="small muted">
            It reads the document, its results, checks and history through Cruncha. It can't change the document
            yet: it explains, reviews and suggests, and you make the changes.
          </p>
        </div>
      {/if}
      {#each entries as e, i (i)}
        {#if e.type === "user"}
          <div class="user">{e.text}</div>
        {:else if e.type === "agent"}
          <div class="agent md" use:rich={e.text}></div>
        {:else if e.type === "thought"}
          <details class="thought">
            <summary>Thinking</summary>
            <div class="md" use:rich={e.text}></div>
          </details>
        {:else if e.type === "tool"}
          {@const key = `${i}:${e.id}`}
          <div class="tool {e.status}" class:outside={outside(e.kind)}>
            <button class="tool-head" onclick={() => (open.has(key) ? open.delete(key) : open.add(key))}>
              <span class="ticon" aria-hidden="true">{toolIcon[e.kind] ?? "◇"}</span>
              <span class="ttitle">{shortTitle(e.title)}</span>
              <span class="tstatus">{e.status === "running" ? "…" : e.status === "failed" ? "failed" : e.status === "pending" ? "waiting" : ""}</span>
            </button>
            {#if open.has(key)}
              {#each e.output as o, j (j)}
                {#if "Text" in o}
                  <pre class="tout">{o.Text}</pre>
                {:else if "Diff" in o}
                  <div class="diff">
                    <div class="small muted">{o.Diff.path}</div>
                    {#if o.Diff.old_text}<pre class="del">{o.Diff.old_text}</pre>{/if}
                    <pre class="add">{o.Diff.new_text}</pre>
                  </div>
                {:else}
                  <pre class="tout muted">(terminal output)</pre>
                {/if}
              {/each}
              {#if e.output.length === 0}<p class="small muted">No output.</p>{/if}
            {/if}
          </div>
        {:else}
          <div class="notice" class:err={e.error}>{e.text}</div>
        {/if}
      {/each}
      {#if view?.busy && !reading}<div class="working"><span class="spinner" aria-hidden="true"></span> Working…</div>{/if}
    </div>

    {#if view?.permissions.length && !reading}
      {#each view.permissions as p (p.id)}
        {@const tool = toolOf(p)}
        <div class="permission" class:outside={tool && outside(tool.kind)}>
          <div><strong>{shortTitle(p.title)}</strong></div>
          {#if tool && outside(tool.kind)}
            <p class="small">This acts outside the document: files or commands on this computer. Cruncha's tools never need it.</p>
          {/if}
          <div class="choices">
            {#each p.options as o (o.id)}
              <button class:primary={o.kind === "allowOnce"} class:danger={o.kind.startsWith("reject")} onclick={() => answer(p, o.id)}>{o.label}</button>
            {/each}
          </div>
        </div>
      {/each}
    {/if}

    {#if error}<div class="notice err">{error} <button class="link" onclick={() => (error = null)}>Dismiss</button></div>{/if}

    {#if !reading}
      {#if view?.agentId && running}
        <div class="prompt">
          <textarea
            bind:value={prompt}
            {onkeydown}
            rows="3"
            placeholder={view.ready ? "Ask about the document… (Enter sends, Shift+Enter for a new line, Esc stops)" : "Starting…"}
            disabled={!view.ready}
          ></textarea>
          <div class="row">
            <button class="link small" onclick={() => void act(() => api.agentStop()).then((v) => v && (view = v))}>Stop agent</button>
            <span class="spacer"></span>
            {#if view.busy}
              <button onclick={() => void act(() => api.agentCancel()).then((v) => v && (view = v))}>■ Stop</button>
            {:else}
              <button class="primary" disabled={!view.ready || !prompt.trim()} onclick={send}>Send</button>
            {/if}
          </div>
        </div>
      {:else}
        <div class="start">
          <label for="agent-choice" class="small muted">Agent</label>
          <select id="agent-choice" bind:value={choice} onchange={() => void refreshKnown()}>
            {#each catalog as a (a.id)}
              <option value={a.id}>{a.name}</option>
            {/each}
          </select>
          <label for="agent-model" class="small muted">Model</label>
          <div class="modelpick">
            {#if choiceModels.length}
              <select id="agent-model" value={startModel} onchange={(e) => void pickModel((e.currentTarget as HTMLSelectElement).value)}>
                <option value="">The agent's default</option>
                {#each choiceModels as m (m.value)}
                  <option value={m.value}>{m.group ? `${m.group} · ` : ""}{m.name}</option>
                {/each}
                {#if startModel && !choiceModels.some((m) => m.value === startModel)}
                  <option value={startModel}>{startModel}</option>
                {/if}
              </select>
            {:else}
              <span class="small muted">{startModel || "The agent's default"}</span>
            {/if}
            <button class="link small" disabled={loadingModels} onclick={() => void loadModels()}>
              {loadingModels ? "Asking the agent…" : choiceModels.length ? "Refresh" : "Load models"}
            </button>
          </div>
          <button class="primary" onclick={start}>Start</button>
          <p class="small muted">
            {catalog.find((a) => a.id === choice)?.note ?? ""}. The adapters run with <code>npx</code> (Node.js).
            <button class="link" onclick={() => onsettings("agents")}>Agent settings…</button> ·
            <button class="link" onclick={() => onsettings("external")}>Use an agent outside Cruncha…</button>
          </p>
        </div>
      {/if}
    {/if}
  {/if}
</aside>

<style>
  .elsewhere { margin: 0; padding: 6px 12px; background: var(--accent-bg); color: var(--text); border-bottom: 1px solid var(--border); }
  .panel {
    display: flex; flex-direction: column; min-height: 0; height: 100%; border-left: 1px solid var(--border);
    background: var(--panel); font-size: 13.5px;
  }
  header { display: flex; align-items: center; gap: 6px; padding: 8px 10px; border-bottom: 1px solid var(--border); flex-wrap: wrap; }
  .who { color: var(--muted); }
  .opt { max-width: 11em; font-size: 12px; padding: 1px 4px; }
  .usage { font-size: 11.5px; padding: 1px 8px; border-radius: 10px; background: var(--sunken); color: var(--muted); cursor: help; }
  .spacer { flex: 1; }
  .icon { border: none; background: none; padding: 2px 6px; font-size: 14px; }
  .plan { margin: 0; padding: 6px 10px 6px 28px; border-bottom: 1px solid var(--border); font-size: 12.5px; }
  .plan li.done { color: var(--muted); text-decoration: line-through; }
  .plan li.active { font-weight: 600; }
  .log { flex: 1; overflow: auto; padding: 10px; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
  .empty { margin: auto 0; text-align: center; padding: 0 12px; }
  .user {
    align-self: flex-end; max-width: 90%; white-space: pre-wrap; padding: 6px 10px; border-radius: 10px 10px 2px 10px;
    background: var(--select); border: 1px solid var(--select-border);
  }
  .agent { line-height: 1.5; }
  .md :global(p) { margin: 0.3em 0; }
  .md :global(pre) { background: var(--sunken); padding: 6px 8px; border-radius: 4px; overflow-x: auto; font: 12px/1.45 var(--mono); }
  .md :global(code) { font: 12.5px var(--mono); }
  .md :global(table) { border-collapse: collapse; font-size: 12.5px; }
  .md :global(th), .md :global(td) { border-bottom: 1px solid var(--border); padding: 2px 8px; text-align: left; }
  .md :global(ul), .md :global(ol) { padding-left: 1.4em; margin: 0.3em 0; }
  .md :global(.cellref) {
    border: none; background: var(--select); color: var(--accent); padding: 0 3px; border-radius: 3px;
    font: inherit; cursor: pointer; text-decoration: underline dotted;
  }
  .thought summary { cursor: pointer; color: var(--muted); font-size: 12.5px; }
  .thought .md { color: var(--muted); font-size: 12.5px; padding-left: 10px; border-left: 2px solid var(--border); }
  .tool { border: 1px solid var(--border); border-radius: 6px; background: var(--bg); }
  .tool.failed { border-color: var(--danger); }
  .tool.outside { border-color: var(--warn); }
  .tool-head { display: flex; gap: 6px; align-items: center; width: 100%; border: none; background: none; padding: 4px 8px; text-align: left; font-size: 12.5px; }
  .ticon { color: var(--muted); width: 1em; }
  .ttitle { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-family: var(--mono); }
  .tstatus { color: var(--muted); font-size: 11.5px; }
  .tout, .diff pre { margin: 0 8px 8px; padding: 6px 8px; background: var(--sunken); border-radius: 4px; font: 11.5px/1.45 var(--mono); white-space: pre-wrap; max-height: 260px; overflow: auto; }
  .diff .del { background: var(--danger-bg); }
  .diff .add { background: color-mix(in srgb, var(--accent) 10%, transparent); }
  .diff > div { margin: 0 8px; }
  .notice { color: var(--muted); font-size: 12.5px; }
  .notice.err { color: var(--danger); }
  .working { color: var(--muted); font-size: 12.5px; display: flex; gap: 6px; align-items: center; }
  .permission { margin: 0 10px 8px; padding: 8px 10px; border: 1px solid var(--accent); border-radius: 6px; background: var(--bg); }
  .permission.outside { border-color: var(--warn); background: var(--warn-bg); }
  .permission p { margin: 4px 0; }
  .choices { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 6px; }
  .prompt { border-top: 1px solid var(--border); padding: 8px 10px; }
  .prompt textarea { width: 100%; resize: vertical; font: inherit; min-height: 3.2em; }
  .row { display: flex; align-items: center; gap: 8px; margin-top: 4px; }
  .start { border-top: 1px solid var(--border); padding: 10px; display: grid; grid-template-columns: auto 1fr; gap: 6px 8px; align-items: center; }
  .start > .primary { grid-column: 2; justify-self: end; }
  .modelpick { display: flex; gap: 8px; align-items: center; min-width: 0; }
  .modelpick select { flex: 1; min-width: 0; }
  .start p { grid-column: 1 / -1; margin: 2px 0 0; }
  .reading { padding: 6px 10px; background: var(--sunken); border-bottom: 1px solid var(--border); }
  .past { flex: 1; overflow: auto; padding: 10px; display: flex; flex-direction: column; gap: 6px; }
  .past-head { display: flex; justify-content: space-between; align-items: center; }
  .conv { display: flex; flex-direction: column; align-items: flex-start; gap: 2px; text-align: left; padding: 6px 8px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg); }
  .spinner {
    width: 10px; height: 10px; border: 2px solid var(--border); border-top-color: var(--accent); border-radius: 50%;
    animation: spin 0.8s linear infinite; display: inline-block;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
