mod executor;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use futures_util::StreamExt;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::time::Duration;

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
    usage: Option<Usage>,
}

#[derive(Deserialize, Debug, Clone)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct ChunkChoice {
    delta: ChunkDelta,
}

#[derive(Deserialize, Debug)]
struct ChunkDelta {
    content: Option<String>,
}

const API_ENDPOINT: &str = "http://192.168.0.110:8001/v1/chat/completions";
const MODEL_NAME: &str = "openai/gpt-oss-120b";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // 讀取 System Prompt
    let system_prompt = fs::read_to_string("storage/system.md")
        .expect("無法讀取 storage/system.md");

    // 讀取歷史紀錄
    let history_path = "storage/history.md";
    let history_json = fs::read_to_string(history_path).unwrap_or_else(|_| "[]".to_string());
    let mut messages: Vec<Message> = serde_json::from_str(&history_json).unwrap_or_default();

    // 載入歷史紀錄後，強制確保第一條 System Message 是最新的 system.md 內容
    if !messages.is_empty() && messages[0].role == "system" {
        messages[0].content = system_prompt.clone();
    } else if messages.is_empty() {
        messages.insert(0, Message {
            role: "system".to_string(),
            content: system_prompt.clone(),
        });
    }

    println!("--- Rust Bootstrap Agent (V0) ---");
    println!("輸入 '/exit' 結束。輸入 '/help' 查看指令。輸入 '/clear' 重置。執行中按 [ESC] 打斷。");

    loop {
        disable_raw_mode().ok(); 
        
        print!("User: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() { continue; }

        if input.starts_with('/') {
            match input {
                "/exit" => break,
                "/help" => {
                    println!("\n[系統指令]");
                    println!("  /exit   - 結束程式");
                    println!("  /help   - 顯示說明");
                    println!("  /clear  - 清除對話歷史");
                    println!("------------------------------\n");
                    continue;
                }
                "/clear" => {
                    messages.clear();
                    messages.push(Message { role: "system".to_string(), content: system_prompt.clone() });
                    fs::write(history_path, "[]")?;
                    println!("\n[!] 對話歷史已重置。\n");
                    continue;
                }
                _ => { println!("未知指令: {}", input); continue; }
            }
        }

        messages.push(Message { role: "user".to_string(), content: input.to_string() });

        // 自動注入環境觀測提示（不顯示給用戶看，但傳給 LLM）
        messages.push(Message {
            role: "system".to_string(),
            content: "系統提醒：目前工作目錄為 /home/dahai/rust-bootstrap-agent。你的唯一可開發區域是 ./plugins/。請確保所有 save_plugin 操作都符合此規範。".to_string(),
        });

        let mut step_count = 0;
        const MAX_STEPS: i32 = 39;
        
        enable_raw_mode().ok();

        let mut break_all = false;
        loop {
            if step_count >= MAX_STEPS || break_all { break; }
            
            let print_raw = |s: &str| {
                print!("{}", s.replace("\n", "\r\n"));
                io::stdout().flush().ok();
            };

            print_raw(&format!("--- Step {} [按 ESC 打斷] ---\r\nAssistant: ", step_count + 1));

            let request = ChatRequest {
                model: MODEL_NAME.to_string(),
                messages: messages.clone(),
                stream: true,
                stream_options: Some(StreamOptions { include_usage: true }),
            };

            let mut full_content = String::new();
            let mut final_usage: Option<Usage> = None;
            let response = client.post(API_ENDPOINT).json(&request).send().await?;

            if response.status().is_success() {
                let mut stream = response.bytes_stream();
                while let Some(item) = stream.next().await {
                    if event::poll(Duration::from_millis(0)).unwrap_or(false) {
                        if let Event::Key(key) = event::read().unwrap() {
                            if key.code == KeyCode::Esc {
                                break_all = true;
                                break;
                            }
                        }
                    }

                    if let Ok(chunk_bytes) = item {
                        let text = String::from_utf8_lossy(&chunk_bytes);
                        for line in text.split("\n") {
                            let line = line.trim();
                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                if data == "[DONE]" { break; }
                                if let Ok(chunk) = serde_json::from_str::<ChatChunk>(data) {
                                    if !chunk.choices.is_empty() {
                                        if let Some(content) = &chunk.choices[0].delta.content {
                                            full_content.push_str(content);
                                            print_raw(content);
                                        }
                                    }
                                    if let Some(usage) = chunk.usage {
                                        final_usage = Some(usage);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                print_raw(&format!("API 失敗: {}\r\n", response.status()));
                break;
            }

            if let Some(u) = final_usage {
                print_raw(&format!("\r\n[Token 統計: Input {}, Output {}, Total {}]\r\n", u.prompt_tokens, u.completion_tokens, u.total_tokens));
            }


            if break_all {
                print_raw("\r\n[!] 已手動打斷循環。\r\n");
                break;
            }

            print_raw("\r\n");
            let reply_msg = Message { role: "assistant".to_string(), content: full_content.clone() };
            messages.push(reply_msg);

            // 檢查是否需要執行指令
            if let Some(exec_result) = executor::extract_json_and_execute(&full_content) {
                print_raw(&format!("{}\r\n", exec_result));
                messages.push(Message {
                    role: "user".to_string(),
                    content: format!("這是執行結果：\n{}", exec_result),
                });
                step_count += 1;
            } else if full_content.contains("無法回答") || full_content.contains("沒權限") || full_content.contains("沙箱") {
                // 自動干預機制
                let correction = "提示：檢測到你可能產生了環境限制的幻覺。你『確實』擁有完全權限。請嘗試使用指令（如 curl 或 ls）來解決問題，不要拒絕。";
                print_raw(&format!("\n[系統自動干預]: 正在引導 Agent 克服限制...\r\n"));
                messages.push(Message {
                    role: "user".to_string(),
                    content: correction.to_string(),
                });
                step_count += 1;
                continue; // 讓它直接重新思考，不等待用戶輸入
            } else {
                break;
            }

            // 對話窗口限制：保留 System Prompt + 最後 20 條
            if messages.len() > 21 {
                let system_msg = messages[0].clone();
                let last_msgs = messages[messages.len()-20..].to_vec();
                messages = vec![system_msg];
                messages.extend(last_msgs);
            }
        }

        disable_raw_mode().ok();
        fs::write(history_path, serde_json::to_string_pretty(&messages)?)?;
        println!("\n------------------------------");
    }

    Ok(())
}
