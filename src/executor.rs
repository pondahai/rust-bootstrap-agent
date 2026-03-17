use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Command;
use std::fs;
use std::path::Path;

/// 模仿 Barbara 的 Tool 結構
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value, // JSON Schema
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentAction {
    pub thought: String,      // 強制要求思考過程
    pub action: String,
    pub args: Value,
}

/// 獲取所有內建與動態工具的規格 (OpenAI 相容)
pub fn get_tools_spec() -> Value {
    let mut tools = vec![
        json!({
            "type": "function",
            "function": {
                "name": "execute_command",
                "description": "執行 Linux 系統指令以獲取即時資訊或操作環境。",
                "parameters": {
                    "type": "object",
                    "properties": { "args": { "type": "string", "description": "完整指令" } },
                    "required": ["args"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "save_script",
                "description": "儲存 Python 或 Bash 腳本到 ./scripts/ 目錄。",
                "parameters": {
                    "type": "object",
                    "properties": { 
                        "name": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["name", "content"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "promote_skill",
                "description": "將經驗證的腳本移至 ./plugins/ 並內化為原生工具。",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "script_name": { "type": "string" },
                        "skill_name": { "type": "string" },
                        "description": { "type": "string" }
                    },
                    "required": ["script_name", "skill_name", "description"]
                }
            }
        })
    ];

    // 動態掃描 plugins/ (自創工具)
    if let Ok(entries) = fs::read_dir("./plugins") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("py") {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if stem == "telegram_bridge" || stem.starts_with("test") { continue; }
                tools.push(json!({
                    "type": "function",
                    "function": {
                        "name": stem,
                        "description": format!("自創原生工具：執行 plugins/{}.py", stem),
                        "parameters": {
                            "type": "object",
                            "properties": { "args": { "type": "string" } }
                        }
                    }
                }));
            }
        }
    }
    Value::Array(tools)
}

pub fn extract_json_and_execute(content: &str) -> Option<String> {
    // 解析包含 thought 的 AgentAction
    let json_str = if let Some(start) = content.find("```json") {
        let block = &content[start + 7..];
        block.find("```").map(|end| block[..end].trim().to_string()).unwrap_or_else(|| block.to_string())
    } else {
        let start = content.find('{').or_else(|| content.find('[')).unwrap_or(usize::MAX);
        let end = content.rfind('}').or_else(|| content.rfind(']')).unwrap_or(0);
        if start < end { content[start..=end].to_string() } else { return None; }
    };

    if let Ok(action) = serde_json::from_str::<AgentAction>(&json_str) {
        println!("🧠 思考: {}", action.thought);
        return perform_action(&action.action, action.args);
    } else if let Ok(actions) = serde_json::from_str::<Vec<AgentAction>>(&json_str) {
        let mut results = Vec::new();
        for a in actions {
            println!("🧠 思考: {}", a.thought);
            if let Some(res) = perform_action(&a.action, a.args) { results.push(res); }
        }
        return Some(results.join("\n\n"));
    }
    None
}

fn perform_action(action_name: &str, args: Value) -> Option<String> {
    match action_name {
        "execute_command" => {
            let cmd = args["args"].as_str().or_else(|| args.as_str()).unwrap_or("");
            println!("🛠️  執行: {}", cmd);
            let output = Command::new("sh").arg("-c").arg(cmd).output().ok()?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                Some(format!("--- 執行成功 ---\n{}", stdout))
            } else {
                Some(format!("--- 執行失敗 ---\nSTDOUT: {}\nSTDERR: {}", stdout, stderr))
            }
        },
        "save_script" => {
            let name = args["name"].as_str()?;
            let content = args["content"].as_str()?;
            fs::write(format!("./scripts/{}", name), content).ok()?;
            Some(format!("✅ 腳本 {} 儲存成功。", name))
        },
        "promote_skill" => {
            let src_name = args["script_name"].as_str()?;
            let skill_name = args["skill_name"].as_str()?;
            let desc = args["description"].as_str()?;
            fs::rename(format!("./scripts/{}", src_name), format!("./plugins/{}", src_name)).ok()?;
            
            let system_path = "./storage/system.md";
            let mut content = fs::read_to_string(system_path).unwrap_or_default();
            content.push_str(&format!("\n- **{}**: {} (plugins/{})\n", skill_name, desc, src_name));
            fs::write(system_path, content).ok();
            
            Some(format!("🚀 進化成功：新工具 '{}' 已上架。", skill_name))
        },
        custom => {
            let py_path = format!("./plugins/{}.py", custom);
            if Path::new(&py_path).exists() {
                println!("🛠️  執行自創工具: {}", custom);
                let arg_str = args["args"].as_str().or_else(|| args.as_str()).unwrap_or("");
                let output = Command::new("python3").arg(&py_path).arg(arg_str).output().ok()?;
                Some(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Some(format!("❌ 未知動作: {}", custom))
            }
        }
    }
}
