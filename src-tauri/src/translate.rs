use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use tauri::{AppHandle, Emitter};

/// A single delta chunk from the DeepSeek streaming response.
#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

/// Translate text using the DeepSeek API with streaming response.
/// Emits "translation-chunk" events to the frontend for each token.
/// Emits "translation-done" when the stream is complete.
/// Emits "translation-error" on failure.
#[tauri::command]
pub async fn translate_text(
    app: AppHandle,
    text: String,
    source_lang: String,
    target_lang: String,
    api_key: String,
    model: String,
) -> Result<(), String> {
    let client = Client::new();

    let system_prompt = if source_lang == "auto" {
        format!(
            "You are a professional translator. Auto-detect the source language of the following text \
             and translate it into {}. Output ONLY the translated text, nothing else. \
             Do not add any explanations, notes, or quotation marks.",
            target_lang
        )
    } else {
        format!(
            "You are a professional translator. Translate the following text from {} to {}. \
             Output ONLY the translated text, nothing else. \
             Do not add any explanations, notes, or quotation marks.",
            source_lang, target_lang
        )
    };

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": text }
        ],
        "temperature": 0.3,
        "stream": true
    });

    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            let msg = format!("Network error: {}", e);
            let _ = app.emit("translation-error", &msg);
            msg
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        let msg = format!("API error ({}): {}", status, error_body);
        let _ = app.emit("translation-error", &msg);
        return Err(msg);
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| {
            let msg = format!("Stream error: {}", e);
            let _ = app.emit("translation-error", &msg);
            msg
        })?;

        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        // SSE format: each line starts with "data: "
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();

            if line.is_empty() {
                continue;
            }

            if line == "data: [DONE]" {
                let _ = app.emit("translation-done", ());
                return Ok(());
            }

            if let Some(json_str) = line.strip_prefix("data: ") {
                if let Ok(chunk) = serde_json::from_str::<StreamChunk>(json_str) {
                    for choice in &chunk.choices {
                        if let Some(content) = &choice.delta.content {
                            if !content.is_empty() {
                                let _ = app.emit("translation-chunk", content.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    let _ = app.emit("translation-done", ());
    Ok(())
}
