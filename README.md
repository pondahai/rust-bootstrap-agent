# Rust Bootstrap Agent (V0)

這是一個極簡但強大的自舉式 Rust Agent，採用 **ReAct (Reasoning and Acting)** 架構。它具備自主思考、執行系統指令、自我修正以及連網診斷的能力。

## 🚀 核心功能
- **ReAct 自主循環**：支援多步推理與行動，單次任務最高支援 39 步自主迭代。
- **多介面支持 (Multi-Interface)**：
  - **本地 CLI**：具備完整打斷、即時串流回饋與 Token 監控。
  - **Telegram Bot**：支援手機遠端操控，具備自動重連機制。
- **共享上下文記憶**：本地與 Telegram 共享同一個對話紀錄，達成無縫銜接。
- **強大的指令執行器**：支援 `execute_command`、`save_script` 與 `save_plugin`。

## ⚙️ 快速管理 (One-Stop Management)
我們提供了 `manage.sh` 腳本，讓您輕鬆管理所有的對話介面：

### 1. 本地互動對話 (Terminal)
這會自動編譯最新程式碼並啟動互動介面。
```bash
chmod +x manage.sh
./manage.sh local
```

### 2. Telegram Bot 啟動 (Background)
背景執行 Python Bridge，持續監聽訊息。
```bash
./manage.sh bot start  # 啟動
./manage.sh bot stop   # 停止
./manage.sh bot logs   # 查看運行日誌
```

### 3. 檢查系統狀態
```bash
./manage.sh status
```

## 📂 目錄結構
```text
.
├── Cargo.toml          # 專案依賴配置
├── src/                # 核心 Rust 原始碼 (大腦)
├── storage/            # 記憶 (history.md) 與準則 (system.md)
├── scripts/            # 任務腳本區
└── plugins/            # 橋接器與進階外掛 (Telegram Bridge)
```

## ⌨️ 系統指令
在 `User:` 提示下或透過 Telegram，您可以使用：
- `/help`：顯示可用指令說明。
- `/clear`：清除對話歷史，重置 Agent 狀態與共享記憶。
- `/exit`：安全退出程式。
- **[ESC]**：(僅本地端) 在 Agent 思考或執行任務中按下，可立即跳回輸入模式。

## ⚠️ 注意事項
- **安全性**：Agent 具備執行 Shell 指令與儲存檔案的權限，請在受信任的環境中使用。
- **API 配置**：在 `src/main.rs` 中修改您的 LLM API 端點與模型名稱。
- **Telegram Token**：請在 `plugins/.env` 中配置您的 Bot Token。
