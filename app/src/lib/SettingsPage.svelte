<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Settings: everything that belongs to this computer rather than to a document. Agents (the
     default, each one's model and connection), provider connections (keys in the Keychain),
     agents outside Cruncha, defaults for documents, the office's report style, Python and
     trusted documents. One scrolling page with a section list, after looking_glass's Settings;
     changes save as you make them, except the history default, which may trim history. -->
<script lang="ts" module>
  export type SettingsSection = "agents" | "connections" | "external" | "documents" | "reports" | "python" | "trusted";
</script>

<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import {
    api,
    DEFAULT_REPORT_STYLE,
    DEFAULT_TEXT_SIZES,
    NO_AGENT_PREF,
    NO_AGENT_SETTINGS,
    type AgentDef,
    type AgentPref,
    type DocInfo,
    type KeySource,
    type Prefs,
    type Preview,
    type ProviderView,
    type PythonEnv,
    type ReportStyle,
    type TextSizes,
  } from "./api";
  import ExternalAgents from "./ExternalAgents.svelte";
  import PruneConfirm from "./PruneConfirm.svelte";
  import RetentionControl from "./RetentionControl.svelte";
  import TextSizeFields from "./TextSizeFields.svelte";
  import { describeRetention, isKeepEverything } from "./format";

  let {
    prefs: initial,
    doc,
    section = "agents",
    onclose,
    onsaved,
    onagentschange,
  }: {
    prefs: Prefs;
    doc: DocInfo | null;
    section?: SettingsSection;
    onclose: () => void;
    onsaved: (p: Prefs) => void;
    onagentschange: (running: boolean) => void;
  } = $props();

  const SECTIONS: { id: SettingsSection; label: string }[] = [
    { id: "agents", label: "Agents" },
    { id: "connections", label: "Connections" },
    { id: "external", label: "Agents outside Cruncha" },
    { id: "documents", label: "New documents" },
    { id: "reports", label: "Reports" },
    { id: "python", label: "Python" },
    { id: "trusted", label: "Trusted documents" },
  ];

  let prefs = $state<Prefs>($state.snapshot(untrack(() => initial)));
  const agents = $derived(prefs.agents ?? NO_AGENT_SETTINGS);
  let catalog = $state<AgentDef[]>([]);
  let providers = $state<ProviderView[]>([]);
  let keys = $state<Record<string, KeySource>>({});
  let python = $state<PythonEnv | null>(null);
  let editing = $state<string>("claude");
  let status = $state<string | null>(null);
  let error = $state<string | null>(null);
  let scroller: HTMLElement;
  let current = $state<SettingsSection>(untrack(() => section));

  onMount(() => {
    void (async () => {
      try {
        prefs = await api.getPrefs(); // the freshest copy (the agent panel may have added models)
        catalog = await api.agentCatalog();
        providers = await api.providers();
        editing = agents.defaultAgent && catalog.some((a) => a.id === agents.defaultAgent) ? agents.defaultAgent : (catalog[0]?.id ?? "claude");
        python = await api.pythonEnv();
        await tick();
        go(untrack(() => section), false);
        keys = await api.connectionKeys();
      } catch (e) {
        error = String(e);
      }
    })();
  });

  function go(id: SettingsSection, smooth = true) {
    current = id;
    document.getElementById(`settings-${id}`)?.scrollIntoView({ block: "start", behavior: smooth ? "smooth" : "auto" });
  }

  function onscroll() {
    const top = scroller.getBoundingClientRect().top + 40;
    for (const s of [...SECTIONS].reverse()) {
      const el = document.getElementById(`settings-${s.id}`);
      if (el && el.getBoundingClientRect().top <= top) {
        current = s.id;
        return;
      }
    }
    current = SECTIONS[0].id;
  }

  // ---- Saving: everything but the history default saves shortly after a change. ----

  let timer: ReturnType<typeof setTimeout> | undefined;
  function persist(next: Prefs) {
    prefs = next;
    clearTimeout(timer);
    timer = setTimeout(async () => {
      try {
        await api.setPrefs($state.snapshot(prefs));
        onsaved($state.snapshot(prefs));
        status = "Saved";
        setTimeout(() => (status = status === "Saved" ? null : status), 1500);
      } catch (e) {
        error = String(e);
      }
    }, 400);
  }

  function setAgents(next: typeof agents) {
    persist({ ...$state.snapshot(prefs), agents: next });
  }

  function setPref(id: string, change: Partial<AgentPref>) {
    const pref = { ...NO_AGENT_PREF, ...agents.agents[id], ...change };
    setAgents({ ...agents, agents: { ...agents.agents, [id]: pref } });
  }

  const pref = $derived({ ...NO_AGENT_PREF, ...agents.agents[editing] });
  const editingDef = $derived(catalog.find((a) => a.id === editing));
  const BUILTIN = "looking-glass";
  /** Connections this agent can run on (an adapter only takes its own provider's key). */
  const COMPATIBLE: Record<string, string[] | undefined> = { claude: ["anthropic"], codex: ["openai"], gemini: ["google"] };
  const usable = $derived(
    agents.connections.filter((c) => editing === BUILTIN || (COMPATIBLE[editing] ?? []).includes(c.provider)),
  );
  const modelGroups = $derived.by(() => {
    const groups = new Map<string, typeof pref.knownModels>();
    for (const m of pref.knownModels) {
      const g = m.group ?? "";
      groups.set(g, [...(groups.get(g) ?? []), m]);
    }
    return [...groups.entries()];
  });

  const sizes = $derived({ ...DEFAULT_TEXT_SIZES, ...prefs.textSizes });
  const style = $derived({ ...DEFAULT_REPORT_STYLE, ...prefs.reportStyle });

  function setStyle(change: Partial<ReportStyle>) {
    const next = { ...style, ...change };
    if (!/^#[0-9a-f]{6}$/i.test(next.accent)) next.accent = DEFAULT_REPORT_STYLE.accent;
    persist({ ...$state.snapshot(prefs), reportStyle: next });
  }

  async function chooseLogo() {
    const path = await openDialog({
      title: "Logo for report headers",
      filters: [{ name: "Image", extensions: ["png", "jpg", "jpeg", "svg"] }],
    });
    if (typeof path === "string") setStyle({ logoPath: path });
  }

  // ---- Connections ----

  let adding = $state({ provider: "anthropic", name: "", baseUrl: "", key: "" });
  let replacing = $state<{ id: string; key: string } | null>(null);
  const providerOf = (id: string) => providers.find((p) => p.id === id);
  const KEY_LABEL: Record<KeySource, string> = {
    keychain: "Keychain ✓",
    environment: "Environment ✓",
    missing: "Missing",
    notneeded: "not needed",
  };

  async function attempt(f: () => Promise<void>) {
    error = null;
    try {
      await f();
    } catch (e) {
      error = String(e);
    }
  }

  const addConnection = () =>
    attempt(async () => {
      const a = $state.snapshot(adding);
      const next = await api.connectionAdd(a.provider, a.name, a.baseUrl, a.key);
      prefs = next;
      onsaved(next);
      adding = { provider: a.provider, name: "", baseUrl: "", key: "" };
      keys = await api.connectionKeys();
    });

  const removeConnection = (id: string) =>
    attempt(async () => {
      const next = await api.connectionRemove(id);
      prefs = next;
      onsaved(next);
      keys = await api.connectionKeys();
    });

  const replaceKey = () =>
    attempt(async () => {
      if (!replacing) return;
      await api.connectionSetKey(replacing.id, replacing.key);
      replacing = null;
      keys = await api.connectionKeys();
    });

  // ---- History default (may trim the open document, so it is applied explicitly) ----

  let policy = $state($state.snapshot(untrack(() => initial.defaultRetention)));
  let confirm = $state<Preview | null>(null);
  const policyChanged = $derived(JSON.stringify(policy) !== JSON.stringify(prefs.defaultRetention));

  const applyPolicy = () =>
    attempt(async () => {
      if (!confirm && doc && doc.retention == null && !isKeepEverything(policy)) {
        const p = await api.retentionPreview($state.snapshot(policy));
        if (p.cutoff != null) {
          confirm = p;
          return;
        }
      }
      const next = { ...$state.snapshot(prefs), defaultRetention: $state.snapshot(policy) };
      await api.setPrefs(next);
      prefs = next;
      onsaved(next);
      confirm = null;
      status = "Saved";
    });

  const forgetTrusted = (id: string) =>
    attempt(async () => {
      const next = await api.forgetTrusted(id);
      prefs = next;
      onsaved(next);
    });

  const fileName = (p: string) => p.split(/[\\/]/).pop() ?? p;
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !confirm && onclose()} />

<div class="settings">
  <nav aria-label="Settings sections">
    <h1>Settings</h1>
    {#each SECTIONS as s (s.id)}
      <button class:on={current === s.id} onclick={() => go(s.id)}>{s.label}</button>
    {/each}
    <span class="spacer"></span>
    <p class="small muted">Saved automatically on this computer. A document's own settings are in <em>Document settings</em>.</p>
  </nav>
  <div class="scroll" bind:this={scroller} {onscroll}>
    <div class="column">
      <header>
        <span class="small muted">{status ?? ""}</span>
        <span class="spacer"></span>
        <button class="primary" onclick={onclose}>Done</button>
      </header>
      {#if error}<p class="error">{error}</p>{/if}

      <section id="settings-agents">
        <h2>Agents</h2>
        <p class="note">The agent the Agent panel starts with, and how each one runs. Click an agent to set it up; the dot marks the default.</p>
        <div class="list">
          {#each catalog as a (a.id)}
            <div class="row" class:sel={editing === a.id}>
              <button
                class="dot"
                class:on={agents.defaultAgent === a.id || (!agents.defaultAgent && a.id === catalog[0]?.id)}
                title="Make it the default"
                aria-label="Make {a.name} the default"
                onclick={() => setAgents({ ...agents, defaultAgent: a.id })}
              ></button>
              <button class="name" onclick={() => (editing = a.id)}>
                <span>{a.name}</span>
                <span class="small muted">{a.note}</span>
              </button>
            </div>
          {/each}
        </div>
        {#if editingDef}
          <div class="card">
            <h3>{editingDef.name}</h3>
            <div class="fields">
              <label for="agent-conn">Runs on</label>
              <div>
                <select
                  id="agent-conn"
                  value={pref.connection ?? ""}
                  onchange={(e) => setPref(editing, { connection: (e.currentTarget as HTMLSelectElement).value || null })}
                >
                  <option value="">{editing === BUILTIN ? "Demo mode (no model)" : "Its own login"}</option>
                  {#each usable as c (c.id)}
                    <option value={c.id}>{c.name} ({providerOf(c.provider)?.label ?? c.provider})</option>
                  {/each}
                </select>
                <p class="hint">
                  {#if editing === BUILTIN}
                    The built-in agent runs on a connection's provider and key. It can't use Cruncha's tools yet.
                  {:else if (COMPATIBLE[editing] ?? []).length}
                    Or an API key: add a {COMPATIBLE[editing]!.map((p) => providerOf(p)?.label ?? p).join(" or ")} connection below.
                  {:else}
                    This agent uses its own login.
                  {/if}
                </p>
              </div>
              <label for="agent-model">Model</label>
              <div>
                {#if pref.knownModels.length}
                  <select
                    id="agent-model"
                    value={pref.model ?? ""}
                    onchange={(e) => setPref(editing, { model: (e.currentTarget as HTMLSelectElement).value || null })}
                  >
                    <option value="">The agent's default</option>
                    {#each modelGroups as [group, models] (group)}
                      {#if group}
                        <optgroup label={group}>
                          {#each models as m (m.value)}<option value={m.value}>{m.name}</option>{/each}
                        </optgroup>
                      {:else}
                        {#each models as m (m.value)}<option value={m.value}>{m.name}</option>{/each}
                      {/if}
                    {/each}
                    {#if pref.model && !pref.knownModels.some((m) => m.value === pref.model)}
                      <option value={pref.model}>{pref.model}</option>
                    {/if}
                  </select>
                {:else}
                  <input
                    id="agent-model"
                    value={pref.model ?? ""}
                    placeholder="The agent's default"
                    onchange={(e) => setPref(editing, { model: (e.currentTarget as HTMLInputElement).value.trim() || null })}
                  />
                {/if}
                <p class="hint">
                  {pref.knownModels.length
                    ? "Selected when a session starts; the panel's menu changes it for one session."
                    : "Start this agent once in the Agent panel and its models appear here. Or type a model id."}
                </p>
              </div>
            </div>
          </div>
        {/if}
      </section>

      <section id="settings-connections">
        <h2>Connections</h2>
        <p class="note">
          Provider accounts for agents to run on, instead of their own logins. Keys are stored in the macOS Keychain,
          never in Cruncha's files, and handed to an agent only when it starts. A key in Cruncha's own environment
          (for example <code>ANTHROPIC_API_KEY</code>) wins over the Keychain.
        </p>
        {#if agents.connections.length}
          <table>
            <thead><tr><th>Name</th><th>Provider</th><th>Key</th><th></th></tr></thead>
            <tbody>
              {#each agents.connections as c (c.id)}
                <tr>
                  <td>{c.name}{#if c.baseUrl}<div class="small muted mono">{c.baseUrl}</div>{/if}</td>
                  <td>{providerOf(c.provider)?.label ?? c.provider}</td>
                  <td class:missing={keys[c.id] === "missing"}>{keys[c.id] ? KEY_LABEL[keys[c.id]] : "…"}</td>
                  <td class="acts">
                    {#if providerOf(c.provider)?.keyVar}
                      <button class="link" onclick={() => (replacing = { id: c.id, key: "" })}>Replace key</button>
                    {/if}
                    <button class="link" onclick={() => void removeConnection(c.id)}>Remove</button>
                  </td>
                </tr>
                {#if replacing?.id === c.id}
                  <tr>
                    <td colspan="4">
                      <form class="inline" onsubmit={(e) => (e.preventDefault(), void replaceKey())}>
                        <input type="password" autocomplete="off" bind:value={replacing.key} placeholder="New key" />
                        <button type="submit" class="primary" disabled={!replacing.key.trim()}>Save key</button>
                        <button type="button" class="link" onclick={() => (replacing = null)}>Cancel</button>
                      </form>
                    </td>
                  </tr>
                {/if}
              {/each}
            </tbody>
          </table>
        {/if}
        <form class="card add" onsubmit={(e) => (e.preventDefault(), void addConnection())}>
          <h3>Add a connection</h3>
          <div class="fields">
            <label for="c-provider">Provider</label>
            <select id="c-provider" bind:value={adding.provider}>
              {#each providers as p (p.id)}
                <option value={p.id}>{p.label}{p.keyVar ? ` · ${p.keyVar}` : ""}</option>
              {/each}
            </select>
            <label for="c-name">Name</label>
            <input id="c-name" bind:value={adding.name} placeholder={providerOf(adding.provider)?.label ?? ""} />
            <label for="c-url">Base URL</label>
            <input id="c-url" bind:value={adding.baseUrl} placeholder={providerOf(adding.provider)?.keyVar ? "optional (the provider's own)" : "http://localhost:11434/v1"} />
            {#if providerOf(adding.provider)?.keyVar}
              <label for="c-key">API key</label>
              <input id="c-key" type="password" autocomplete="off" bind:value={adding.key} placeholder="stored in the Keychain" />
            {/if}
          </div>
          <div class="right"><button type="submit">Add</button></div>
        </form>
      </section>

      <section id="settings-external">
        <h2>Agents outside Cruncha</h2>
        <ExternalAgents onchange={onagentschange} />
      </section>

      <section id="settings-documents">
        <h2>New documents</h2>
        <p class="note">Defaults for every document that doesn't set its own in Document settings.</p>
        <h3>Text sizes</h3>
        <TextSizeFields
          sizes={sizes}
          onchange={(t: TextSizes) => persist({ ...$state.snapshot(prefs), textSizes: t })}
        />
        <h3>History</h3>
        <p class="hint">
          Every change is saved as a version. Now: {describeRetention(prefs.defaultRetention).toLowerCase()}.
        </p>
        <RetentionControl bind:policy disabled={confirm != null} />
        {#if confirm && doc}
          <PruneConfirm preview={confirm} docName={doc.name} />
        {/if}
        <div class="right">
          {#if confirm}
            <button onclick={() => (confirm = null)}>Back</button>
            <button class="danger" onclick={applyPolicy}>Apply and trim</button>
          {:else}
            <button disabled={!policyChanged} onclick={() => (policy = $state.snapshot(prefs.defaultRetention))}>Revert</button>
            <button class="primary" disabled={!policyChanged} onclick={applyPolicy}>Apply</button>
          {/if}
        </div>
      </section>

      <section id="settings-reports">
        <h2>Reports</h2>
        <p class="note">The office's look, the same for every document. Each document's title block is in its Document settings.</p>
        <div class="fields">
          <label for="rs-company">Company</label>
          <input id="rs-company" value={style.company} placeholder="Shown in the page header" onchange={(e) => setStyle({ company: (e.currentTarget as HTMLInputElement).value })} />
          <label for="rs-logo">Logo</label>
          <span class="logo">
            <button id="rs-logo" onclick={chooseLogo}>{style.logoPath ? "Change…" : "Choose…"}</button>
            {#if style.logoPath}
              <span class="small muted path" title={style.logoPath}>{fileName(style.logoPath)}</span>
              <button class="link" onclick={() => setStyle({ logoPath: null })}>Remove</button>
            {/if}
            <span class="small muted">replaces the company name</span>
          </span>
          <label for="rs-accent">Accent</label>
          <span><input id="rs-accent" type="color" value={style.accent} onchange={(e) => setStyle({ accent: (e.currentTarget as HTMLInputElement).value })} /> <span class="small muted">headings and rules</span></span>
          <label for="rs-paper">Paper</label>
          <select id="rs-paper" value={style.paper} onchange={(e) => setStyle({ paper: (e.currentTarget as HTMLSelectElement).value as ReportStyle["paper"] })}>
            <option value="us-letter">US Letter</option>
            <option value="a4">A4</option>
          </select>
        </div>
      </section>

      <section id="settings-python">
        <h2>Python</h2>
        {#if python}
          <p>
            {#if python.available}
              <strong>Ready</strong> · environment <code>{python.label}</code>
            {:else}
              <strong>Not set up.</strong> Open a document with Python cells and choose <em>Set up Python</em>;
              Cruncha installs its own Python 3.13 with NumPy, SciPy, Polars, pint, SymPy and matplotlib (about 500 MB, once).
            {/if}
          </p>
          <p class="hint mono">{python.interpreter ?? python.base}</p>
        {/if}
      </section>

      <section id="settings-trusted">
        <h2>Trusted documents</h2>
        <p class="note">
          Documents allowed to run their Python cells on this computer: ones created here, and ones you chose to trust.
          Forgetting one makes it ask again when it opens.
        </p>
        {#if (prefs.trusted ?? []).length === 0}
          <p class="hint">None.</p>
        {:else}
          <ul class="trusted">
            {#each prefs.trusted ?? [] as id (id)}
              {@const path = prefs.trustedPaths?.[id]}
              <li>
                <span class="tname" title={path ?? id}>{path ? fileName(path) : `document ${id.slice(-8)}`}</span>
                {#if path}<span class="small muted mono tpath">{path}</span>{/if}
                <button class="link" onclick={() => void forgetTrusted(id)}>Forget</button>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    </div>
  </div>
</div>

<style>
  .settings {
    position: fixed; inset: 0; z-index: 25; background: var(--bg);
    display: grid; grid-template-columns: 220px 1fr; min-height: 0;
  }
  nav {
    border-right: 1px solid var(--border); background: var(--panel); padding: 16px 10px;
    display: flex; flex-direction: column; gap: 2px;
  }
  nav h1 { font-size: 18px; margin: 4px 8px 12px; }
  nav button { border: none; background: none; text-align: left; padding: 6px 10px; border-radius: 6px; color: var(--muted); }
  nav button.on { background: var(--sunken); color: var(--text); font-weight: 600; }
  nav .spacer { flex: 1; }
  nav p { margin: 8px; }
  .scroll { overflow: auto; min-height: 0; }
  .column { max-width: 760px; padding: 12px 32px 80px; }
  header { position: sticky; top: 0; display: flex; align-items: center; padding: 8px 0; background: var(--bg); z-index: 1; }
  .spacer { flex: 1; }
  section { padding: 12px 0 20px; border-bottom: 1px solid var(--border); scroll-margin-top: 48px; }
  h2 { font-size: 16px; margin: 0 0 4px; }
  h3 { font-size: 14px; margin: 14px 0 6px; }
  .card h3 { margin-top: 0; }
  .note, .hint { color: var(--muted); font-size: 13px; margin: 2px 0 8px; }
  .hint { margin: 4px 0 0; font-size: 12px; }
  .list { border: 1px solid var(--border); border-radius: 8px; background: var(--panel); overflow: hidden; }
  .row { display: flex; align-items: center; gap: 4px; padding: 2px 8px; border-top: 1px solid var(--border); }
  .row:first-child { border-top: none; }
  .row.sel { background: var(--select); }
  .dot {
    width: 14px; height: 14px; border-radius: 50%; border: 2px solid var(--border); background: none; padding: 0;
    flex: none;
  }
  .dot.on { border-color: var(--accent); background: radial-gradient(var(--accent) 45%, transparent 50%); }
  .name { flex: 1; border: none; background: none; text-align: left; display: grid; padding: 6px; }
  .card { border: 1px solid var(--border); border-radius: 8px; padding: 12px 14px; margin-top: 10px; background: var(--panel); }
  .fields { display: grid; grid-template-columns: 120px 1fr; gap: 8px 12px; align-items: start; }
  .fields > label { padding-top: 5px; }
  .fields input:not([type="color"]), .fields select { width: 100%; max-width: 420px; }
  .right { display: flex; justify-content: flex-end; gap: 8px; margin-top: 10px; }
  table { width: 100%; border-collapse: collapse; background: var(--panel); border: 1px solid var(--border); border-radius: 8px; }
  th { text-align: left; font-size: 11px; text-transform: uppercase; letter-spacing: 0.04em; color: var(--muted); padding: 6px 10px; }
  td { padding: 6px 10px; border-top: 1px solid var(--border); vertical-align: top; }
  td.missing { color: var(--danger); }
  .acts { text-align: right; white-space: nowrap; }
  .acts .link + .link { margin-left: 10px; }
  .inline { display: flex; gap: 8px; align-items: center; }
  .inline input { flex: 1; }
  .mono { font-family: var(--mono); font-size: 12px; overflow-wrap: anywhere; }
  .logo { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .path { max-width: 16em; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .trusted { list-style: none; padding: 0; margin: 0; display: grid; gap: 4px; }
  .trusted li { display: grid; grid-template-columns: auto 1fr auto; gap: 10px; align-items: baseline; }
  .tpath { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .error { color: var(--danger); }
  @media (max-width: 720px) {
    .settings { grid-template-columns: 1fr; }
    nav { display: none; }
    .column { padding: 12px 16px 60px; }
  }
</style>
