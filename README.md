# Rust Bootstrap Agent (V0)

這是一個極簡但強大的自舉式 Rust Agent，採用 **ReAct (Reasoning and Acting)** 架構。它具備自主思考、執行系統指令、自我修正以及連網診斷的能力。

## 🚀 核心功能
- **ReAct 自主循環**：支援多步推理與行動，單次任務最高支援 39 步自主迭代。
- **即時串流輸出 (Streaming)**：即時顯示 Agent 的推理過程（Thought），讓思考透明化。
- **Token 消耗監控**：每個步驟結束後自動顯示輸入/輸出 Token 統計，精確掌握資源消耗。
- **強大的指令執行器**：支援 `execute_command`、`save_script` (任務導向) 與 `save_plugin` (技能導向)。
- **動態終端控制**：
  - **自動切換模式**：對話時為標準模式，執行時自動切換至 Raw 模式。
  - **即時打斷**：執行過程中按下 `[ESC]` 可立即中止當前循環。

## 🛠️ 技術棧
...
- **流處理**: `futures-util`

## 📂 目錄結構
```text
.
├── Cargo.toml          # 專案依賴配置
├── src/                # 核心 Rust 原始碼
├── storage/            # 記憶與準則存放區
├── scripts/            # 任務腳本區：存放一次性、任務導向的腳本
└── plugins/            # 進化實驗室：存放可重複利用、擴充能力的工具
```

## ⚙️ 快速開始

### 1. 安裝環境
確保系統已安裝 Rust 工具鏈：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### 2. 設定 API
在 `src/main.rs` 中修改您的內網 API 端點：
```rust
const API_ENDPOINT: &str = "http://192.168.0.110:8001/v1/chat/completions";
const MODEL_NAME: &str = "openai/gpt-oss-120b";
```

### 3. 編譯與執行
```bash
cargo run
```

## ⌨️ 系統指令
在 `User:` 提示下輸入以下指令：
- `/help`：顯示可用指令說明。
- `/clear`：清除對話歷史，重置 Agent 狀態（當 Agent 陷入邏輯死循環時非常有效）。
- `/exit`：安全退出程式並恢復終端機設定。
- **[ESC]**：在 Agent 思考或執行任務中按下，可立即跳回輸入模式。

## ⚠️ 注意事項
- **安全性**：Agent 具備執行 Shell 指令的能力，請在受信任的環境中使用。
- **環境**：推薦在 Linux 或 WSL (Windows Subsystem for Linux) 環境下執行。
