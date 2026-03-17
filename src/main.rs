mod executor;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::{self, Write};
use std::env;
use futures_util::StreamExt;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Value>,
    stream: bool,
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
        // --- Telegram 模式 (單次驅動任務) ---
        messages.push(Message { 
            role: "system".to_string(), 
            content: "用戶正透過手機對話。嚴禁編造。請遵循：思考 -> 行動 -> 觀察 -> 總結。".to_string() 
        });
        messages.push(Message { role: "user".to_string(), content: input });
        process_and_respond(&client, &mut messages, history_path, true, &api_endpoint, &model_name).await?;
        return Ok(());
    }

    // --- 本地互動模式 ---
    let mut rl = DefaultEditor::new()?;
    println!("--- 🧠 Rust Bootstrap Agent (Barbara Loop Activated) ---");
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
            println!("\n[!] 已重置對話。\n");
            continue;
        }
        messages.push(Message { role: "user".to_string(), content: input });
        process_and_respond(&client, &mut messages, history_path, false, &api_endpoint, &model_name).await?;
        println!("\n------------------------------");
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

    loop {
        if step_count >= MAX_STEPS { break; }
        let print_raw = |s: &str| { if !is_silent { print!("{}", s.replace("\n", "\r\n")); io::stdout().flush().ok(); } };

        if !is_silent { println!("--- Step {} [Thinking...] ---", step_count + 1); }

        let request = ChatRequest {
            model: model.to_string(),
            messages: messages.clone(),
            tools: Some(executor::get_tools_spec()), // 每次循環重新掃描動態工具
            stream: true,
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
        } else {
            print_raw(&format!("\n❌ API 錯誤: {}\n", response.status()));
            break;
        }

        if !is_silent { println!(); }
        messages.push(Message { role: "assistant".to_string(), content: full_content.clone() });

        // 嘗試提取並執行工具
        if let Some(exec_result) = executor::extract_json_and_execute(&full_content) {
            if !is_silent { println!("🔍 Observation:\n{}", exec_result); }
            
            // 將結果作為 Observation 餵回給 LLM (模仿 Barbara 的遞迴)
            messages.push(Message { 
                role: "user".to_string(), 
                content: format!("Observation (物理執行結果):\n{}", exec_result) 
            });
            step_count += 1;
            // 繼續循環，讓 LLM 決定下一步
        } else {
            // 沒有工具調用，代表對話結束
            if is_silent { println!("{}", full_content); }
            break;
        }
    }

    fs::write(history_path, serde_json::to_string_pretty(&messages)?)?;
    Ok(())
}
