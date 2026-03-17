mod executor;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::env;
use futures_util::StreamExt;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::time::Duration;
use rustyline::DefaultEditor;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    stream_options: Option<StreamOptions>,
}

#[derive(Serialize)]
struct StreamOptions {
    include_usage: bool,
}

#[derive(Deserialize, Debug)]
struct ChatChunk {
    choices: Vec<ChunkChoice>,
}

#[derive(Deserialize, Debug)]
struct ChunkChoice {
    delta: ChunkDelta,
}

#[derive(Deserialize, Debug)]
struct ChunkDelta {
    content: Option<String>,
}

const DEFAULT_API_ENDPOINT: &str = "http://192.168.0.110:8001/v1/chat/completions";
const DEFAULT_MODEL_NAME: &str = "openai/gpt-oss-120b";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let args: Vec<String> = env::args().collect();
    let single_input = if args.len() > 1 { Some(args[1..].join(" ")) } else { None };

    let api_endpoint = env::var("LLM_API_URL").unwrap_or_else(|_| DEFAULT_API_ENDPOINT.to_string());
    let model_name = env::var("LLM_MODEL_NAME").unwrap_or_else(|_| DEFAULT_MODEL_NAME.to_string());

    let system_prompt = fs::read_to_string("storage/system.md").expect("無法讀取 storage/system.md");
    let history_path = "storage/history.md";
    let history_json = fs::read_to_string(history_path).unwrap_or_else(|_| "[]".to_string());
    let mut messages: Vec<Message> = serde_json::from_str(&history_json).unwrap_or_default();

    if !messages.is_empty() && messages[0].role == "system" {
        messages[0].content = system_prompt.clone();
    } else {
        messages.insert(0, Message { role: "system".to_string(), content: system_prompt.clone() });
    }

    if let Some(input) = single_input {
        messages.push(Message { 
            role: "system".to_string(), 
            content: "注意：你目前在 Telegram (手機)。嚴禁編造假數據。如果執行指令失敗，請誠實告知。".to_string() 
        });
        messages.push(Message { role: "user".to_string(), content: input });
        process_and_respond(&client, &mut messages, history_path, true, &api_endpoint, &model_name).await?;
        return Ok(());
    }

    let mut rl = DefaultEditor::new()?;
    println!("--- 🧠 Rust Bootstrap Agent ---");
    loop {
        disable_raw_mode().ok(); 
        let readline = rl.readline("User: ");
        let input = match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                line.trim().to_string()
            },
            Err(_) => break,
        };
        if input.is_empty() { continue; }
        if input == "/exit" { break; }
        if input == "/clear" {
            messages.clear();
            messages.push(Message { role: "system".to_string(), content: system_prompt.clone() });
            fs::write(history_path, "[]")?;
            println!("\n[!] 已重置。\n");
            continue;
        }
        messages.push(Message { role: "user".to_string(), content: input });
        process_and_respond(&client, &mut messages, history_path, false, &api_endpoint, &model_name).await?;
    }
    Ok(())
}

async fn process_and_respond(
    client: &Client, 
    messages: &mut Vec<Message>, 
    history_path: &str,
    is_silent: bool,
    api_url: &str,
    model: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let mut step_count = 0;
    const MAX_STEPS: i32 = 10;

    if !is_silent { enable_raw_mode().ok(); }

    loop {
        if step_count >= MAX_STEPS { break; }
        let print_raw = |s: &str| { if !is_silent { print!("{}", s.replace("\n", "\r\n")); io::stdout().flush().ok(); } };

        let request = ChatRequest {
            model: model.to_string(),
            messages: messages.clone(),
            stream: true,
            stream_options: None,
        };

        let mut full_content = String::new();
        let response = client.post(api_url).json(&request).send().await?;

        if response.status().is_success() {
            let mut stream = response.bytes_stream();
            while let Some(item) = stream.next().await {
                if let Ok(chunk_bytes) = item {
                    let text = String::from_utf8_lossy(&chunk_bytes);
                    for line in text.split("\n").map(|l| l.trim()).filter(|l| l.starts_with("data: ")) {
                        let data = &line[6..];
                        if data == "[DONE]" { break; }
                        if let Ok(chunk) = serde_json::from_str::<ChatChunk>(data) {
                            if let Some(content) = chunk.choices.get(0).and_then(|c| c.delta.content.as_ref()) {
                                full_content.push_str(content);
                                if !is_silent { print_raw(content); }
                            }
                        }
                    }
                }
            }
        }

        if !is_silent { print_raw("\r\n"); }
        messages.push(Message { role: "assistant".to_string(), content: full_content.clone() });

        if let Some(exec_result) = executor::extract_json_and_execute(&full_content) {
            if !is_silent { println!("{}", exec_result); }
            
            // 強力攔截：如果看到失敗標記，強制注入否定命令
            let feedback = if exec_result.contains("!!! 指令失敗 !!!") {
                format!("【硬性攔截】指令執行失敗了！原始錯誤回傳如下：\n{}\n請停止一切編造與模擬行為。誠實告訴用戶執行失敗的原因，嚴禁提供任何數值！", exec_result)
            } else {
                format!("這是物理執行的真實輸出：\n{}", exec_result)
            };
            
            messages.push(Message { role: "user".to_string(), content: feedback });
            step_count += 1;
        } else {
            if is_silent { println!("{}", full_content); }
            break;
        }
    }

    if !is_silent { disable_raw_mode().ok(); }
    fs::write(history_path, serde_json::to_string_pretty(&messages)?)?;
    Ok(())
}
