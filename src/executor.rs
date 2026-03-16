use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentAction {
    pub action: String,
    pub args: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveFileArgs {
    pub name: String,
    pub content: String,
}

pub fn extract_json_and_execute(content: &str) -> Option<String> {
    let json_str = if let Some(start) = content.find("```json") {
        let block_content = &content[start + 7..];
        if let Some(end) = block_content.find("```") {
            block_content[..end].trim().to_string()
        } else {
            block_content.trim().to_string()
        }
    } else if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            content[start..=end].trim().to_string()
        } else {
            return None;
        }
    } else {
        return None;
    };

    if let Ok(action) = serde_json::from_str::<AgentAction>(&json_str) {
        return match action.action.as_str() {
            "execute_command" => {
                println!("--- 執行指令: {} ---", action.args);
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(&action.args)
                    .output();

                match output {
                    Ok(o) => {
                        let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                        let status = o.status.code().map_or("Unknown".to_string(), |c| c.to_string());
                        Some(format!("--- 執行結果 ---\nExit Status: {}\nSTDOUT:\n{}\nSTDERR:\n{}", status, stdout, stderr))
                    }
                    Err(e) => Some(format!("執行錯誤: {}", e)),
                }
            },
            "save_script" => {
                if let Ok(save_args) = serde_json::from_str::<SaveFileArgs>(&action.args) {
                    let path = format!("./scripts/{}", save_args.name);
                    println!("--- 儲存任務腳本: {} ---", path);
                    match fs::write(&path, &save_args.content) {
                        Ok(_) => Some(format!("成功將腳本儲存至 {}", path)),
                        Err(e) => Some(format!("儲存腳本失敗: {}", e)),
                    }
                } else {
                    Some("save_script 參數格式錯誤".to_string())
                }
            },
            "save_plugin" => {
                if let Ok(save_args) = serde_json::from_str::<SaveFileArgs>(&action.args) {
                    let path = format!("./plugins/{}", save_args.name);
                    println!("--- 儲存通用插件: {} ---", path);
                    match fs::write(&path, &save_args.content) {
                        Ok(_) => Some(format!("成功將插件儲存至 {}", path)),
                        Err(e) => Some(format!("儲存插件失敗: {}", e)),
                    }
                } else {
                    Some("save_plugin 參數格式錯誤".to_string())
                }
            },
            _ => Some(format!("未知的行動類型: {}", action.action)),
        };
    }
    None
}
