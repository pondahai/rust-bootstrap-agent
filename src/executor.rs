use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentAction {
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
        return perform_action(action);
    } else if let Ok(actions) = serde_json::from_str::<Vec<AgentAction>>(&json_str) {
        let mut results = Vec::new();
        for action in actions {
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
                    
                    // 強化失敗報告
                    if status != "0" || stdout.contains("Bad Request") || stdout.contains("400") {
                        Some(format!("【!!! 指令失敗 !!!】\nExit Code: {}\nSTDOUT (包含錯誤):\n{}\nSTDERR:\n{}", status, stdout, stderr))
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
                println!("--- 儲存任務腳本: {} ---", path);
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
                match fs::write(system_path, system_content) {
                    Ok(_) => Some(format!("✅ 進化成功！新技能 '{}' 已永久內化。", args.skill_name)),
                    Err(e) => Some(format!("檔案移動成功，但更新系統手冊失敗: {}", e)),
                }
            } else { Some("promote_skill 參數格式錯誤".to_string()) }
        },
        "save_plugin" => {
            if let Ok(args) = serde_json::from_value::<SaveFileArgs>(action.args) {
                let path = format!("./plugins/{}", args.name);
                match fs::write(&path, &args.content) {
                    Ok(_) => Some(format!("成功將插件儲存至 {}", path)),
                    Err(e) => Some(format!("儲存插件失敗: {}", e)),
                }
            } else { Some("save_plugin 參數格式錯誤".to_string()) }
        },
        _ => Some(format!("未知的行動類型: {}", action.action)),
    }
}
