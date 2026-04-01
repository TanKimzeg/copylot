use crate::app::config::AppConfig;

const TEMPLATE: &str = r#"
你是一个翻译助手，专门帮助用户将文本从一种语言翻译成简体中文。
请根据用户提供的文本进行翻译，并确保翻译结果准确、流畅且符合语法和文化习惯。
注意：请只提供翻译结果，不要包含任何解释、评论或原文。
对原文可能出现的专业术语、俚语或文化特定的表达进行适当的翻译，以确保译文在中文环境中易于理解。
"#;

pub async fn invoke(app: &tauri::AppHandle, input: &str) -> String {
    use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
    use openai::Credentials;

    let cfg = AppConfig::read_with_app(app);

    let model = cfg
        .translation_model
        .as_deref()
        .unwrap_or("deepseek-chat")
        .to_string();

    let api_key = cfg.translation_api_key.unwrap_or_default();

    // 优先使用配置；否则根据 model 估一个默认
    let base_url = cfg
        .translation_base_url
        .unwrap_or_else(|| match model.as_str() {
            "deepseek-chat" => "https://api.deepseek.com/v1".to_string(),
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
            content: Some(TEMPLATE.to_string()),
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
        .create()
        .await
    {
        Ok(chat_completion) => {
            let ret_message = chat_completion.choices.first().unwrap().message.clone();
            log::debug!("translation response message: {:?}", ret_message);
            return ret_message.content.unwrap().trim().to_string();
        }
        Err(e) => {
            log::error!("translation invoke failed: {e:?}");
            "（翻译失败）".to_string()
        }
    }
}
