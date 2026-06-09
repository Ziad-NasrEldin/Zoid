import { invoke } from "@tauri-apps/api/core";

export type ManagedProviderStatus = "draft" | "validated" | "invalid" | "applied";

export type ManagedProvider = {
  id: string;
  displayName: string;
  providerType: string;
  providerId: string;
  apiKeyEnv: string;
  defaultModel: string;
  modelOptions: string[];
  baseUrl: string;
  status: ManagedProviderStatus;
  applied: boolean;
  keyStored: boolean;
  createdAt: string;
  updatedAt: string;
  lastValidatedAt: string;
  lastAppliedAt: string;
  lastError: string;
};

export type ProviderInput = {
  id?: string;
  displayName: string;
  providerType: string;
  providerId: string;
  apiKeyEnv: string;
  apiKey?: string;
  defaultModel: string;
  modelOptions: string[];
  baseUrl: string;
};

export type ProviderValidationResult = {
  ok: boolean;
  status: ManagedProviderStatus;
  message: string;
  availableModels: string[];
  validatedAt: string;
};

export type ProviderApplyResult = {
  ok: boolean;
  message: string;
  provider: ManagedProvider;
  configPath: string;
  envPath: string;
};

export type ProviderKeyReveal = {
  providerId: string;
  apiKey: string;
};

export type ProviderTemplate = {
  value: string;
  label: string;
  providerId: string;
  apiKeyEnv: string;
  defaultModel: string;
  models: string[];
  baseUrl: string;
  note: string;
};

export const PROVIDER_TEMPLATES: ProviderTemplate[] = [
  {
    value: "google-gemini",
    label: "Google Gemini",
    providerId: "google",
    apiKeyEnv: "GOOGLE_API_KEY",
    defaultModel: "gemini-2.5-pro",
    models: ["gemini-2.5-pro", "gemini-2.5-flash", "gemini-2.5-flash-lite", "gemini-2.0-flash", "gemini-2.0-flash-lite", "gemma-3-27b-it", "gemma-3-12b-it", "gemma-3-4b-it", "gemma-3-1b-it"],
    baseUrl: "",
    note: "Google AI Studio / Gemini API key. Gemma models are listed beside Gemini models when available.",
  },
  {
    value: "openrouter",
    label: "OpenRouter",
    providerId: "openrouter",
    apiKeyEnv: "OPENROUTER_API_KEY",
    defaultModel: "openai/gpt-5.5",
    models: ["anthropic/claude-opus-4.8", "anthropic/claude-sonnet-4.6", "anthropic/claude-haiku-4.5", "openai/gpt-5.5", "openai/gpt-5.5-pro", "openai/gpt-5.4-mini", "google/gemini-3-pro-preview", "google/gemini-3.1-pro-preview", "deepseek/deepseek-v4-pro", "qwen/qwen3.7-max"],
    baseUrl: "",
    note: "Broad model router; model IDs use provider/model names.",
  },
  {
    value: "anthropic",
    label: "Anthropic",
    providerId: "anthropic",
    apiKeyEnv: "ANTHROPIC_API_KEY",
    defaultModel: "claude-sonnet-4.6",
    models: ["claude-opus-4.8", "claude-opus-4.7", "claude-sonnet-4.6", "claude-sonnet-4.5", "claude-haiku-4.5"],
    baseUrl: "",
    note: "Anthropic API key provider.",
  },
  {
    value: "openai",
    label: "OpenAI / ChatGPT",
    providerId: "openai",
    apiKeyEnv: "OPENAI_API_KEY",
    defaultModel: "gpt-5.5",
    models: ["gpt-5.5", "gpt-5.5-pro", "gpt-5.4", "gpt-5.4-pro", "gpt-5.4-mini", "gpt-5.3", "gpt-5.3-mini", "gpt-5-mini"],
    baseUrl: "",
    note: "OpenAI API key provider. The live Hermes model cache is merged when available so this dropdown stays current after provider discovery/validation.",
  },
  {
    value: "deepseek",
    label: "DeepSeek",
    providerId: "deepseek",
    apiKeyEnv: "DEEPSEEK_API_KEY",
    defaultModel: "deepseek-chat",
    models: ["deepseek-chat", "deepseek-reasoner"],
    baseUrl: "",
    note: "DeepSeek direct API key provider.",
  },
  {
    value: "custom-openai-compatible",
    label: "Custom OpenAI-compatible",
    providerId: "custom",
    apiKeyEnv: "CUSTOM_API_KEY",
    defaultModel: "custom-model",
    models: ["custom-model"],
    baseUrl: "https://api.example.com/v1",
    note: "Use for providers that expose an OpenAI-compatible API. Base URL is required.",
  },
];

export function providerTemplateByValue(value: string) {
  return PROVIDER_TEMPLATES.find((template) => template.value === value) ?? PROVIDER_TEMPLATES[0];
}

export async function listManagedProviders(): Promise<ManagedProvider[]> {
  try {
    return await invoke<ManagedProvider[]>("list_managed_providers");
  } catch {
    return [];
  }
}

export async function saveManagedProvider(provider: ProviderInput): Promise<ManagedProvider> {
  return await invoke<ManagedProvider>("save_managed_provider", { provider });
}

export async function validateManagedProvider(providerId: string): Promise<ProviderValidationResult> {
  return await invoke<ProviderValidationResult>("validate_managed_provider", { providerId });
}

export async function applyManagedProvider(providerId: string): Promise<ProviderApplyResult> {
  return await invoke<ProviderApplyResult>("apply_managed_provider", { providerId });
}

export async function revealManagedProviderKey(providerId: string): Promise<ProviderKeyReveal> {
  return await invoke<ProviderKeyReveal>("reveal_managed_provider_key", { providerId });
}
