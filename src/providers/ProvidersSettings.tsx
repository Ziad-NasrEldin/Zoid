import { Eye, EyeOff, KeyRound, Plus, RefreshCw, Save, ShieldCheck, Wand2 } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { GlobalDropdown } from "../ui/GlobalDropdown";
import {
  applyManagedProvider,
  listManagedProviders,
  ManagedProvider,
  ProviderInput,
  providerTemplateByValue,
  PROVIDER_TEMPLATES,
  revealManagedProviderKey,
  saveManagedProvider,
  validateManagedProvider,
} from "./providerClient";

type ProvidersSettingsProps = {
  availableModels: Record<string, string[]>;
  onProvidersChanged: (providers: ManagedProvider[]) => void;
  onSelectMainProvider: (providerId: string, modelName: string) => void;
};

const blankKeyPlaceholder = "••••••••••••••••";

function providerStatusLabel(provider: ManagedProvider) {
  if (provider.applied) return "Applied";
  if (provider.status === "validated") return "Validated";
  if (provider.status === "invalid") return "Needs attention";
  return "Draft";
}

function mergeLiveModels(templateModels: string[], liveModels: string[] | undefined) {
  return Array.from(new Set([...(liveModels ?? []), ...templateModels].filter(Boolean)));
}

function newProviderDraft(availableModels: Record<string, string[]> = {}): ProviderInput {
  const template = providerTemplateByValue("google-gemini");
  const modelOptions = mergeLiveModels(template.models, availableModels[template.providerId]);
  return {
    displayName: template.label,
    providerType: template.value,
    providerId: template.providerId,
    apiKeyEnv: template.apiKeyEnv,
    apiKey: "",
    defaultModel: modelOptions[0] ?? template.defaultModel,
    modelOptions,
    baseUrl: template.baseUrl,
  };
}

export function ProvidersSettings({ availableModels, onProvidersChanged, onSelectMainProvider }: ProvidersSettingsProps) {
  const [providers, setProviders] = useState<ManagedProvider[]>([]);
  const [draft, setDraft] = useState<ProviderInput>(() => newProviderDraft(availableModels));
  const [status, setStatus] = useState("Loading managed providers…");
  const [saving, setSaving] = useState(false);
  const [visibleKeyProviderId, setVisibleKeyProviderId] = useState<string>("");
  const [revealedKeys, setRevealedKeys] = useState<Record<string, string>>({});
  const onProvidersChangedRef = useRef(onProvidersChanged);

  useEffect(() => {
    onProvidersChangedRef.current = onProvidersChanged;
  }, [onProvidersChanged]);

  useEffect(() => {
    let active = true;
    listManagedProviders().then((loaded) => {
      if (!active) return;
      setProviders(loaded);
      onProvidersChangedRef.current(loaded);
      setStatus(loaded.length ? `Loaded ${loaded.length} managed provider${loaded.length === 1 ? "" : "s"}.` : "No managed providers yet. Add Google Gemini to start.");
    });
    return () => { active = false; };
  }, []);

  const template = providerTemplateByValue(draft.providerType);
  const liveTemplateModels = availableModels[draft.providerId];
  const effectiveDraftModels = useMemo(() => mergeLiveModels(draft.modelOptions ?? [], liveTemplateModels), [draft.modelOptions, liveTemplateModels]);
  const modelOptions = useMemo(() => Array.from(new Set([...effectiveDraftModels, draft.defaultModel].filter(Boolean))).map((model) => ({ value: model, label: model })), [draft.defaultModel, effectiveDraftModels]);

  function updateDraft(partial: Partial<ProviderInput>) {
    setDraft((current) => ({ ...current, ...partial }));
  }

  function changeTemplate(value: string) {
    const selected = providerTemplateByValue(value);
    const modelOptions = mergeLiveModels(selected.models, availableModels[selected.providerId]);
    setDraft({
      id: draft.id,
      displayName: selected.label,
      providerType: selected.value,
      providerId: selected.providerId,
      apiKeyEnv: selected.apiKeyEnv,
      apiKey: "",
      defaultModel: modelOptions[0] ?? selected.defaultModel,
      modelOptions,
      baseUrl: selected.baseUrl,
    });
  }

  function editProvider(provider: ManagedProvider) {
    setDraft({
      id: provider.id,
      displayName: provider.displayName,
      providerType: provider.providerType,
      providerId: provider.providerId,
      apiKeyEnv: provider.apiKeyEnv,
      apiKey: "",
      defaultModel: provider.defaultModel,
      modelOptions: mergeLiveModels(provider.modelOptions, availableModels[provider.providerId]),
      baseUrl: provider.baseUrl,
    });
    setStatus(`Editing ${provider.displayName}. Leave API key blank to keep the stored key.`);
  }

  async function refreshProviders(nextStatus?: string) {
    const loaded = await listManagedProviders();
    setProviders(loaded);
    onProvidersChanged(loaded);
    if (nextStatus) setStatus(nextStatus);
    return loaded;
  }

  async function saveDraft() {
    setSaving(true);
    setStatus("Saving provider metadata and secure key entry…");
    try {
      const saved = await saveManagedProvider(draft);
      let nextStatus = `Saved ${saved.displayName}. Apply/sync when ready.`;
      if ((draft.apiKey ?? "").trim()) {
        const validation = await validateManagedProvider(saved.id);
        nextStatus = validation.message;
      }
      await refreshProviders(nextStatus);
      onSelectMainProvider(saved.providerId, saved.defaultModel);
      setDraft({ ...newProviderDraft(availableModels), id: undefined });
    } catch (error) {
      setStatus(`Provider save failed: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setSaving(false);
    }
  }

  async function testProvider(provider: ManagedProvider) {
    setStatus(`Testing ${provider.displayName}…`);
    try {
      const result = await validateManagedProvider(provider.id);
      await refreshProviders(result.message);
    } catch (error) {
      setStatus(`Provider test failed: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  async function applyProvider(provider: ManagedProvider) {
    setStatus(`Applying ${provider.displayName} to Hermes config and .env…`);
    try {
      const result = await applyManagedProvider(provider.id);
      await refreshProviders(result.message);
      onSelectMainProvider(result.provider.providerId, result.provider.defaultModel);
    } catch (error) {
      setStatus(`Apply failed: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  async function toggleReveal(provider: ManagedProvider) {
    if (visibleKeyProviderId === provider.id) {
      setVisibleKeyProviderId("");
      return;
    }
    try {
      const reveal = await revealManagedProviderKey(provider.id);
      setRevealedKeys((current) => ({ ...current, [provider.id]: reveal.apiKey }));
      setVisibleKeyProviderId(provider.id);
    } catch (error) {
      setStatus(`Reveal failed: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  return (
    <section className="profile-section providers-settings-section" aria-label="Provider management settings">
      <div className="profile-section-title"><p>04</p><h3>Providers</h3><span>Manage local Zoid providers, API keys, model presets, validation, and manual Hermes sync/apply.</span></div>
      <div className="providers-manager-grid">
        <article className="provider-editor-card">
          <div className="provider-card-heading">
            <KeyRound size={18} aria-hidden="true" />
            <div><h4>{draft.id ? "Edit provider" : "Add provider"}</h4><p>Keys are stored through the native backend; Zoid only shows them masked unless you reveal.</p></div>
          </div>
          <div className="provider-form-grid">
            <label className="profile-field"><span>Provider template</span><GlobalDropdown label="Provider template" onChange={changeTemplate} options={PROVIDER_TEMPLATES.map((option) => ({ value: option.value, label: option.label }))} value={draft.providerType} /><small>{template.note}</small></label>
            <label className="profile-field"><span>Display name</span><input value={draft.displayName} onChange={(event) => updateDraft({ displayName: event.target.value })} /></label>
            <label className="profile-field"><span>Hermes provider id</span><input value={draft.providerId} onChange={(event) => updateDraft({ providerId: event.target.value })} /></label>
            <label className="profile-field"><span>API key env</span><input value={draft.apiKeyEnv} onChange={(event) => updateDraft({ apiKeyEnv: event.target.value })} /></label>
            <label className="profile-field profile-field--wide"><span>API key</span><input autoComplete="off" placeholder={draft.id ? "Leave blank to keep stored key" : blankKeyPlaceholder} type="password" value={draft.apiKey ?? ""} onChange={(event) => updateDraft({ apiKey: event.target.value })} /><small>Best secure option for this macOS app: API key secret lives in Keychain; apply/sync writes the needed env var to Hermes .env.</small></label>
            <label className="profile-field"><span>Default model</span><GlobalDropdown label="Default model" onChange={(nextValue) => updateDraft({ defaultModel: nextValue })} options={modelOptions} value={draft.defaultModel} /><small>No manual model typing for presets; custom providers can edit the model list later.</small></label>
            <label className="profile-field"><span>Base URL</span><input value={draft.baseUrl} onChange={(event) => updateDraft({ baseUrl: event.target.value })} placeholder="Only needed for custom providers" /></label>
          </div>
          <div className="provider-action-row"><button type="button" onClick={() => setDraft(newProviderDraft(availableModels))}><Plus size={14} aria-hidden="true" /> New</button><button type="button" onClick={saveDraft} disabled={saving}><Save size={14} aria-hidden="true" /> Save provider</button></div>
        </article>

        <div className="provider-list" role="list">
          {providers.length === 0 ? <p className="repo-empty-state">No providers saved yet.</p> : providers.map((provider) => (
            <article className={`provider-card provider-card--${provider.status}`} key={provider.id} role="listitem">
              <div className="provider-card-heading">
                <ShieldCheck size={18} aria-hidden="true" />
                <div><h4>{provider.displayName}</h4><p>{provider.providerId} · {provider.defaultModel}</p></div>
                <span className="provider-status-badge">{providerStatusLabel(provider)}</span>
              </div>
              <dl className="provider-meta-grid">
                <div><dt>API key</dt><dd>{visibleKeyProviderId === provider.id ? revealedKeys[provider.id] : `${provider.apiKeyEnv}=${blankKeyPlaceholder}`}</dd></div>
                <div><dt>Models</dt><dd>{provider.modelOptions.slice(0, 5).join(", ")}{provider.modelOptions.length > 5 ? "…" : ""}</dd></div>
                <div><dt>Last check</dt><dd>{provider.lastValidatedAt ? new Date(Number(provider.lastValidatedAt) || provider.lastValidatedAt).toLocaleString() : "Not tested"}</dd></div>
                <div><dt>Last apply</dt><dd>{provider.lastAppliedAt ? new Date(Number(provider.lastAppliedAt) || provider.lastAppliedAt).toLocaleString() : "Not applied"}</dd></div>
              </dl>
              {provider.lastError ? <p className="provider-error-note" role="alert">{provider.lastError}</p> : null}
              <div className="provider-action-row">
                <button type="button" onClick={() => editProvider(provider)}>Edit</button>
                <button type="button" onClick={() => toggleReveal(provider)}>{visibleKeyProviderId === provider.id ? <EyeOff size={14} aria-hidden="true" /> : <Eye size={14} aria-hidden="true" />} {visibleKeyProviderId === provider.id ? "Hide key" : "Reveal key"}</button>
                <button type="button" onClick={() => testProvider(provider)}><Wand2 size={14} aria-hidden="true" /> Test</button>
                <button type="button" onClick={() => applyProvider(provider)}><RefreshCw size={14} aria-hidden="true" /> Apply / sync</button>
              </div>
            </article>
          ))}
        </div>
      </div>
      <p className="profile-save-status" role="status">{status}</p>
    </section>
  );
}
