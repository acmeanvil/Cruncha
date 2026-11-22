//! Provider profiles.
//!
//! Each profile is an API style, a base URL, where to find the API key, an
//! optional default model, and the quirks that differ between providers.
//! Checked against each provider's docs on 2026-11-14/15; any field can be
//! overridden with flags.

pub const NAMES: &[&str] = &[
    "anthropic",
    "openai",
    "google",
    "huggingface",
    "deepseek",
    "qwen",
    "nvidia",
    "tinker",
    "ollama",
    "lmstudio",
    "openai-compatible",
];

/// The wire protocol a provider speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Api {
    /// OpenAI Chat Completions (and the many servers that copy it).
    OpenAiChat,
    /// Anthropic's Messages API (Claude), used natively.
    Anthropic,
}

/// How to list a provider's models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelList {
    /// `GET {base}/models`, OpenAI style (`data[].id`).
    OpenAi,
    /// `GET {base}/v1/models` with Anthropic headers.
    Anthropic,
    /// Google's native `GET /v1beta/models` (the OpenAI-compatible endpoint
    /// doesn't document listing).
    Google,
}

#[derive(Debug, Clone)]
pub struct Provider {
    pub id: String,
    pub label: String,
    pub api: Api,
    pub base_url: String,
    /// The model to use; empty until one is chosen (or listed, for local servers).
    pub model: String,
    pub api_key_env: Option<String>,
    /// Send earlier turns' `reasoning_content` back with each request.
    /// DeepSeek requires this whenever tools are present (400 otherwise).
    pub echo_reasoning: bool,
    /// Ask for token usage in the stream (`stream_options.include_usage`).
    pub stream_usage: bool,
    pub tools: bool,
    /// Runs on this machine (no key, models come from the server).
    pub local: bool,
    /// Models offered before the server's own list arrives.
    pub suggested: Vec<ModelInfo>,
    list: ModelList,
}

/// A model as offered to the user: grouped by family for the pickers.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    /// Family, e.g. "Opus" for Claude or "Gemini 3" for Google.
    pub group: Option<String>,
    /// Context window in tokens, when the provider reports it.
    pub context: Option<u64>,
}

impl ModelInfo {
    fn plain(id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: id.to_string(),
            group: family_from_id(id),
            context: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct Overrides {
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
    pub no_tools: bool,
}

struct Profile {
    label: &'static str,
    api: Api,
    base: Option<&'static str>,
    key: Option<&'static str>,
    model: Option<&'static str>,
    suggested: &'static [&'static str],
    echo_reasoning: bool,
    stream_usage: bool,
    list: ModelList,
}

fn profile(name: &str) -> Option<Profile> {
    let openai = |label, base, key| Profile {
        label,
        api: Api::OpenAiChat,
        base,
        key,
        model: None,
        suggested: &[],
        echo_reasoning: false,
        stream_usage: false,
        list: ModelList::OpenAi,
    };
    Some(match name {
        // Native Messages API. Default model per Anthropic's current guidance.
        "anthropic" => Profile {
            api: Api::Anthropic,
            model: Some("claude-opus-5-5"),
            list: ModelList::Anthropic,
            ..openai(
                "Claude",
                Some("https://api.anthropic.com"),
                Some("ANTHROPIC_API_KEY"),
            )
        },
        // Model names change often; they come from GET /models.
        "openai" => Profile {
            stream_usage: true,
            ..openai(
                "OpenAI",
                Some("https://api.openai.com/v1"),
                Some("OPENAI_API_KEY"),
            )
        },
        // https://ai.google.dev/gemini-api/docs/openai
        "google" => Profile {
            list: ModelList::Google,
            ..openai(
                "Google",
                Some("https://generativelanguage.googleapis.com/v1beta/openai"),
                Some("GEMINI_API_KEY"),
            )
        },
        // Hugging Face Inference Providers router.
        "huggingface" => openai(
            "Hugging Face",
            Some("https://router.huggingface.co/v1"),
            Some("HF_TOKEN"),
        ),
        // https://api-docs.deepseek.com/guides/thinking_mode
        "deepseek" => Profile {
            model: Some("deepseek-flash"),
            suggested: &["deepseek-flash", "deepseek-v4-pro"],
            echo_reasoning: true,
            stream_usage: true,
            ..openai(
                "DeepSeek",
                Some("https://api.deepseek.com"),
                Some("DEEPSEEK_API_KEY"),
            )
        },
        // Alibaba Cloud Model Studio, international (Singapore) endpoint.
        "qwen" => Profile {
            model: Some("qwen3-coder-plus"),
            suggested: &["qwen3-coder-plus", "qwen3-coder-next"],
            stream_usage: true,
            ..openai(
                "Qwen",
                Some("https://dashscope-intl.aliyuncs.com/compatible-mode/v1"),
                Some("DASHSCOPE_API_KEY"),
            )
        },
        // build.nvidia.com hosted NIM; self-hosted NIM: --base-url http://host:8000/v1
        "nvidia" => Profile {
            model: Some("nvidia/nemotron-3-super-120b-a12b"),
            suggested: &[
                "nvidia/nemotron-3-super-120b-a12b",
                "nvidia/nemotron-3-ultra-550b-a55b",
                "nvidia/nemotron-3-nano-30b-a3b",
            ],
            stream_usage: true,
            ..openai(
                "NVIDIA",
                Some("https://integrate.api.nvidia.com/v1"),
                Some("NVIDIA_API_KEY"),
            )
        },
        // Tinker's OpenAI-compatible endpoint (beta, meant for testing and low
        // traffic). The model is a sampler checkpoint path:
        // tinker://<run-id>:train:0/sampler_weights/<step>
        "tinker" => openai(
            "Thinking Machines",
            Some("https://tinker.thinkingmachines.dev/services/tinker-prod/oai/api/v1"),
            Some("TINKER_API_KEY"),
        ),
        // Local model servers. Models are listed from the server itself.
        "ollama" => Profile {
            stream_usage: true,
            ..openai("Ollama", Some("http://localhost:11434/v1"), None)
        },
        "lmstudio" => Profile {
            stream_usage: true,
            ..openai("LM Studio", Some("http://localhost:1234/v1"), None)
        },
        // Anything else that speaks the API: vLLM, llama.cpp, OpenRouter…
        "openai-compatible" => openai("OpenAI-compatible", None, None),
        _ => return None,
    })
}

impl Provider {
    pub fn new(name: &str, o: Overrides) -> Result<Self, String> {
        let p = profile(name)
            .ok_or_else(|| format!("unknown provider \"{name}\" (known: {})", NAMES.join(", ")))?;
        let base_url = o
            .base_url
            .or(p.base.map(str::to_string))
            .ok_or("--base-url is required for this provider")?;
        let base_url = base_url.trim_end_matches('/').to_string();
        let local = is_local_url(&base_url);
        if name == "tinker" && o.model.is_none() {
            return Err("--model is required: pass a sampler checkpoint path (tinker://…)".into());
        }
        // Without a default, the user picks from the provider's list (local
        // servers pick their first model automatically).
        let model = o.model.or(p.model.map(str::to_string)).unwrap_or_default();
        Ok(Self {
            id: name.to_string(),
            label: if local && name == "openai-compatible" {
                "Local server".to_string()
            } else {
                p.label.to_string()
            },
            api: p.api,
            base_url,
            model,
            api_key_env: o.api_key_env.or(p.key.map(str::to_string)),
            echo_reasoning: p.echo_reasoning,
            stream_usage: p.stream_usage,
            tools: !o.no_tools,
            local,
            suggested: p.suggested.iter().map(|s| ModelInfo::plain(s)).collect(),
            list: p.list,
        })
    }

    pub fn chat_url(&self) -> String {
        match self.api {
            Api::OpenAiChat => format!("{}/chat/completions", self.base_url),
            Api::Anthropic => format!("{}/v1/messages", self.base_url),
        }
    }

    /// Models the provider offers. Blocking; call off the main loop.
    pub fn fetch_models(&self) -> Result<Vec<ModelInfo>, String> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(10)))
            .build()
            .into();
        let key = self.api_key().ok().flatten();
        let get = |url: String, headers: &[(&str, String)]| -> Result<serde_json::Value, String> {
            let mut req = agent.get(url);
            for (k, v) in headers {
                req = req.header(*k, v.as_str());
            }
            req.call()
                .map_err(|e| e.to_string())?
                .body_mut()
                .read_json()
                .map_err(|e| e.to_string())
        };
        let mut models: Vec<ModelInfo> = match self.list {
            ModelList::OpenAi => {
                let auth: Vec<(&str, String)> = key
                    .map(|k| vec![("Authorization", format!("Bearer {k}"))])
                    .unwrap_or_default();
                let body = get(format!("{}/models", self.base_url), &auth)?;
                let mut ids: Vec<String> = body["data"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|m| m["id"].as_str().map(str::to_string))
                    .filter(|id| self.id != "openai" || is_openai_chat_model(id))
                    .collect();
                ids.sort();
                ids.iter().map(|id| ModelInfo::plain(id)).collect()
            }
            ModelList::Anthropic => {
                let headers = vec![
                    ("x-api-key", key.unwrap_or_default()),
                    (
                        "anthropic-version",
                        crate::anthropic::API_VERSION.to_string(),
                    ),
                ];
                let body = get(format!("{}/v1/models?limit=1000", self.base_url), &headers)?;
                body["data"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|m| {
                        let id = m["id"].as_str()?.to_string();
                        let name = m["display_name"].as_str().unwrap_or(&id).to_string();
                        Some(ModelInfo {
                            group: claude_family(&name),
                            context: m["max_input_tokens"].as_u64(),
                            name,
                            id,
                        })
                    })
                    .collect()
            }
            ModelList::Google => {
                let headers = vec![("x-goog-api-key", key.unwrap_or_default())];
                let body = get(
                    "https://generativelanguage.googleapis.com/v1beta/models?pageSize=1000"
                        .to_string(),
                    &headers,
                )?;
                body["models"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|m| {
                        m["supportedGenerationMethods"]
                            .as_array()
                            .is_some_and(|ms| ms.iter().any(|x| x == "generateContent"))
                    })
                    .filter_map(|m| {
                        let id = m["name"]
                            .as_str()?
                            .trim_start_matches("models/")
                            .to_string();
                        Some(ModelInfo {
                            name: m["displayName"].as_str().unwrap_or(&id).to_string(),
                            group: family_from_id(&id),
                            context: m["inputTokenLimit"].as_u64(),
                            id,
                        })
                    })
                    .collect()
            }
        };
        models.dedup_by(|a, b| a.id == b.id);
        Ok(models)
    }

    /// The API key, if this provider uses one. Local servers often don't.
    pub fn api_key(&self) -> Result<Option<String>, String> {
        match &self.api_key_env {
            None => Ok(None),
            Some(var) => match std::env::var(var) {
                Ok(k) if !k.trim().is_empty() => Ok(Some(k.trim().to_string())),
                _ => Err(format!(
                    "{} needs an API key: add one in Settings (it's kept in the Keychain), \
                     or set {var} in the environment",
                    self.label
                )),
            },
        }
    }
}

/// "Claude Opus 5.5" -> "Opus".
fn claude_family(display_name: &str) -> Option<String> {
    let mut words = display_name.split_whitespace();
    match (words.next(), words.next()) {
        (Some("Claude"), Some(family)) => Some(family.to_string()),
        _ => None,
    }
}

/// A family for grouping, from a model id: `org/model` -> `org`;
/// `gemini-3-pro` -> `Gemini 3`; `gpt-5.6-sol` -> `GPT 5.6`.
fn family_from_id(id: &str) -> Option<String> {
    if let Some((org, _)) = id.split_once('/') {
        return Some(org.to_string());
    }
    // Ollama tags (`qwen3:0.6b`) name a size or variant, not a family.
    let base = id.split(':').next().unwrap_or(id);
    let mut parts = base.split('-');
    let first = parts.next()?;
    let name = match first {
        "gpt" => "GPT".to_string(),
        other => {
            let mut c = other.chars();
            c.next()?.to_uppercase().chain(c).collect()
        }
    };
    Some(match parts.next() {
        Some(v) if v.chars().next().is_some_and(|c| c.is_ascii_digit()) => format!("{name} {v}"),
        _ => name,
    })
}

/// OpenAI's list includes embedding, audio and image models; keep the ones
/// that can chat.
fn is_openai_chat_model(id: &str) -> bool {
    const SKIP: &[&str] = &[
        "embedding",
        "tts",
        "whisper",
        "dall-e",
        "image",
        "audio",
        "realtime",
        "moderation",
        "transcribe",
        "search",
        "davinci",
        "babbage",
        "sora",
    ];
    !SKIP.iter().any(|s| id.contains(s))
}

fn is_local_url(url: &str) -> bool {
    let host = url
        .split("://")
        .nth(1)
        .unwrap_or(url)
        .split(['/', ':'])
        .next()
        .unwrap_or("");
    matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]") || host.ends_with(".local")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_and_overrides() {
        let d = Provider::new("deepseek", Overrides::default()).unwrap();
        assert_eq!(d.chat_url(), "https://api.deepseek.com/chat/completions");
        assert!(d.echo_reasoning);
        let q = Provider::new(
            "qwen",
            Overrides {
                model: Some("qwen-plus".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(q.model, "qwen-plus");
        assert!(Provider::new("tinker", Overrides::default()).is_err());
        assert!(Provider::new("openai-compatible", Overrides::default()).is_err());
        let o = Provider::new("ollama", Overrides::default()).unwrap();
        assert!(o.local && o.model.is_empty() && o.api_key_env.is_none());
        assert!(!d.local);
        assert!(Provider::new("nope", Overrides::default()).is_err());

        let a = Provider::new("anthropic", Overrides::default()).unwrap();
        assert_eq!(a.api, Api::Anthropic);
        assert_eq!(a.chat_url(), "https://api.anthropic.com/v1/messages");
        assert_eq!(a.model, "claude-opus-5-5");
        let g = Provider::new("google", Overrides::default()).unwrap();
        assert_eq!(
            g.chat_url(),
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
        );
        assert!(g.model.is_empty() && !g.local);
    }

    #[test]
    fn families() {
        assert_eq!(claude_family("Claude Opus 5.5").as_deref(), Some("Opus"));
        assert_eq!(claude_family("Claude Fable 5.2").as_deref(), Some("Fable"));
        assert_eq!(family_from_id("gemini-3-pro").as_deref(), Some("Gemini 3"));
        assert_eq!(family_from_id("gpt-5.6-sol").as_deref(), Some("GPT 5.6"));
        assert_eq!(family_from_id("qwen3:0.6b").as_deref(), Some("Qwen3"));
        assert_eq!(
            family_from_id("meta-llama/Llama-4").as_deref(),
            Some("meta-llama")
        );
        assert!(!is_openai_chat_model("text-embedding-3-large"));
        assert!(is_openai_chat_model("gpt-5.6-sol"));
    }
}
