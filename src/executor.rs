use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Command;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentAction {
    pub plan: Option<String>,
    pub action: String,
    pub args: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveFileArgs {
    pub name: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PromoteSkillArgs {
    pub script_name: String,
    pub skill_name: String,
    pub description: String,
}

/// 動態掃描 plugins/ 並返回完整的工具規格
pub fn get_tools_spec() -> Value {
    let mut tools = vec![
        json!({
            "type": "function",
            "function": {
                "name": "execute_command",
                "description": "執行 Linux 指令獲取即時資訊。",
                "parameters": {
                    "type": "object",
                    "properties": { "args": { "type": "string" } },
                    "required": ["args"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "save_script",
                "description": "儲存任務專用腳本到 ./scripts/。",
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
                "description": "將已驗證腳本升遷為永久技能/工具。",
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

    // 掃描 plugins 目錄下的所有 .py 檔案，將其作為動態工具加入
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
                        "description": format!("已內化的自創工具：執行 plugins/{}.py", stem),
                        "parameters": {
                            "type": "object",
                            "properties": { "args": { "type": "string", "description": "傳遞給腳本的參數" } }
                        }
                    }
                }));
            }
        }
    }

    Value::Array(tools)
}

pub fn extract_json_and_execute(content: &str) -> Option<String> {
    let json_str = if let Some(start) = content.find("```json") {
        let block_content = &content[start + 7..];
        if let Some(end) = block_content.find("```") {
            block_content[..end].trim().to_string()
        } else {
            block_content.trim().to_string()
        }
    } else {
        let first_brace = content.find('{').unwrap_or(usize::MAX);
        let first_bracket = content.find('[').unwrap_or(usize::MAX);
        let start = std::cmp::min(first_brace, first_bracket);
        if start == usize::MAX { return None; }
        let last_brace = content.rfind('}').unwrap_or(0);
        let last_bracket = content.rfind(']').unwrap_or(0);
        let end = std::cmp::max(last_brace, last_bracket);
        if end <= start { return None; }
        content[start..=end].trim().to_string()
    };

    if let Ok(action) = serde_json::from_str::<AgentAction>(&json_str) {
        if let Some(p) = &action.plan { println!("📋 計畫: {}", p); }
        return perform_action(action);
    } else if let Ok(actions) = serde_json::from_str::<Vec<AgentAction>>(&json_str) {
        let mut results = Vec::new();
        for action in actions {
            if let Some(p) = &action.plan { println!("📋 計畫: {}", p); }
            if let Some(res) = perform_action(action) {
                results.push(res);
            }
        }
        return Some(results.join("\n\n"));
    }
    None
}

fn perform_action(action: AgentAction) -> Option<String> {
    match action.action.as_str() {
        "execute_command" => {
            let cmd = action.args.as_str().unwrap_or("");
            println!("--- 執行指令: {} ---", cmd);
            let output = Command::new("sh").arg("-c").arg(cmd).output();
            match output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                    let status = o.status.code().map_or("Unknown".to_string(), |c| c.to_string());
                    if status != "0" {
                        Some(format!("【!!! 指令失敗 !!!】\nExit Code: {}\nSTDOUT:\n{}\nSTDERR:\n{}", status, stdout, stderr))
                    } else {
                        Some(format!("--- 執行成功 ---\nSTDOUT:\n{}\n", stdout))
                    }
                }
                Err(e) => Some(format!("執行錯誤: {}", e)),
            }
        },
        "save_script" => {
            if let Ok(args) = serde_json::from_value::<SaveFileArgs>(action.args) {
                let path = format!("./scripts/{}", args.name);
                match fs::write(&path, &args.content) {
                    Ok(_) => Some(format!("成功將腳本儲存至 {}", path)),
                    Err(e) => Some(format!("儲存腳本失敗: {}", e)),
                }
            } else { Some("save_script 參數格式錯誤".to_string()) }
        },
        "promote_skill" => {
            if let Ok(args) = serde_json::from_value::<PromoteSkillArgs>(action.args) {
                let src = format!("./scripts/{}", args.script_name);
                let dst = format!("./plugins/{}", args.script_name);
                if let Err(e) = fs::rename(&src, &dst) {
                    return Some(format!("無法移動腳本: {} (確認來源檔案是否存在於 ./scripts/ 中)", e));
                }
                let system_path = "./storage/system.md";
                let mut system_content = fs::read_to_string(system_path).unwrap_or_default();
                if !system_content.contains("## 🧩 已學會的技能") {
                    system_content.push_str("\n\n## 🧩 已學會的技能\n");
                }
                let new_skill = format!("- **{}**: {} (使用 `python plugins/{}`)\n", 
                    args.skill_name, args.description, args.script_name);
                system_content.push_str(&new_skill);
                fs::write(system_path, system_content).ok();
                Some(format!("✅ 進化成功！工具 '{}' 已加入 plugins 並內化為原生外部工具。", args.skill_name))
            } else { Some("promote_skill 參數格式錯誤".to_string()) }
        },
        // 動態處理自創工具 (如果 action 名稱對應到一個 python 檔案)
        custom_action => {
            let py_path = format!("./plugins/{}.py", custom_action);
            if Path::new(&py_path).exists() {
                let args_str = action.args.as_str().unwrap_or("");
                println!("--- 執行自創工具: {} ---", custom_action);
                let output = Command::new("python3").arg(&py_path).arg(args_str).output();
                match output {
                    Ok(o) => {
                        let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                        Some(format!("--- 自創工具 {} 執行結果 ---\n{}", custom_action, stdout))
                    }
                    Err(e) => Some(format!("自創工具執行錯誤: {}", e)),
                }
            } else {
                Some(format!("未知的行動類型: {}", custom_action))
            }
        }
    }
}
