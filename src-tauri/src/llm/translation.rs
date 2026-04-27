use std::sync::LazyLock;

use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use openai::Credentials;

use crate::app::config::AppConfig;
use crate::app::StoreExt;
use tauri::Emitter;

const DEFAULT_MODEL: &str = "deepseek-v4-flash";

static SYSTEM_PROMPT: &str = r#"
你是一个翻译助手，专门帮助用户将文本从一种语言翻译成简体中文。
请根据用户提供的文本进行翻译，并确保翻译结果准确、流畅且符合语法和文化习惯。
注意：请只提供翻译结果，不要包含任何解释、评论或原文。
对原文可能出现的专业术语、俚语或文化特定的表达进行适当的翻译，以确保译文在中文环境中易于理解。
"#;

static SYSTEM_PROMPT_STRING: LazyLock<String> = LazyLock::new(|| SYSTEM_PROMPT.to_string());

#[derive(serde::Serialize, Clone)]
struct TranslationChunkPayload {
    text: String,
}

pub async fn invoke(app: &tauri::AppHandle, input: &str) -> String {
    let cfg = AppConfig::read_with_app(app);

    let model = cfg
        .translation_model
        .as_deref()
        .unwrap_or(DEFAULT_MODEL)
        .to_string();

    let api_key = cfg.translation_api_key.unwrap_or_default();

    // 优先使用配置的 base_url；否则根据 model 推断默认值
    let base_url = cfg
        .translation_base_url
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| match model.as_str() {
            "deepseek-v4-flash" => "https://api.deepseek.com/v1".to_string(),
            _ => "https://api.openai.com/v1".to_string(),
        });

    if api_key.is_empty() {
        log::error!("translation_api_key is empty");
        return "（未配置 API Key）".to_string();
    }

    let credentials = Credentials::new(api_key, base_url);

    let messages = vec![
        ChatCompletionMessage {
            role: ChatCompletionMessageRole::System,
            content: Some(SYSTEM_PROMPT_STRING.clone()),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
        },
        ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: Some(input.to_string()),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    match ChatCompletion::builder(&model, messages)
        .credentials(credentials)
        .create_stream()
        .await
    {
        Ok(mut rx) => {
            let mut full_text = String::new();
            while let Some(delta) = rx.recv().await {
                if let Some(content) = delta.choices.first().and_then(|c| c.delta.content.as_ref())
                {
                    full_text.push_str(content);
                    // 只发送当前 delta（增量），前端自行累积
                    if let Err(e) = app.emit_to(
                        crate::TRANSLATOR_WINDOW_LABEL,
                        "translation-chunk",
                        TranslationChunkPayload {
                            text: content.clone(),
                        },
                    ) {
                        log::warn!("emit translation-chunk failed (window may be closed): {e:?}");
                        break; // 窗口已关闭，无需继续流式传输
                    }
                }
            }
            full_text.trim().to_string()
        }
        Err(e) => {
            log::error!("translation stream failed: {e:?}");
            "（翻译失败）".to_string()
        }
    }
}
