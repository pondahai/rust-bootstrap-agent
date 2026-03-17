# Rust Bootstrap Agent (V0)

這是一個極簡但強大的自舉式 Rust Agent，採用 **ReAct (Reasoning and Acting)** 架構。它具備自主思考、執行系統指令、自我修正以及連網診斷的能力。

## 🚀 核心功能
- **ReAct 自主循環**：支援多步推理與行動，單次任務最高支援 39 步自主迭代。
- **多介面支持 (Multi-Interface)**：
  - **本地 CLI**：具備完整打斷、即時串流回饋與 Token 監控。
  - **Telegram Bot**：支援手機遠端操控，具備自動重連機制。
- **共享上下文記憶**：本地與 Telegram 共享同一個對話紀錄，達成無縫銜接。
- **動態配置 (Runtime Config)**：修改 API 或模型無需重新編譯，即改即用。

## ⚙️ 快速管理 (One-Stop Management)
我們提供了 `manage.sh` 腳本，讓您輕鬆完成所有設定與部署：

### 1. 互動式安裝 (初次啟動)
這會引導您填寫 LLM 與 Telegram 資訊，並自動生成設定檔。
```bash
chmod +x manage.sh
./manage.sh setup
```

### 2. 啟動對話介面
```bash
./manage.sh local      # 啟動本地互動 CLI
./manage.sh bot start  # 背景啟動 Telegram Bot
```

### 3. 查看運行狀態
```bash
./manage.sh status     # 檢查配置與 Bot 運行狀態
./manage.sh bot logs   # 查看 Telegram 運行日誌
```

## 📂 目錄結構
```text
.
├── manage.sh           # 全能管理腳本 (推薦入口)
├── src/                # 核心 Rust 原始碼 (大腦)
├── storage/            # 記憶 (history.md) 與準則 (system.md)
├── plugins/            # 橋接器與進階外掛 (Telegram Bridge)
└── .env                # (自動生成) LLM 配置檔案
```

## ⌨️ 系統指令
在 `User:` 提示下或透過 Telegram，您可以使用：
- `/help`：顯示可用指令說明。
- `/clear`：重置對話歷史與共享記憶。
- `/exit`：安全退出程式。
- **[ESC]**：(僅本地端) 執行任務中按下可立刻打斷循環。

## ⚠️ 安全與安裝細節
- 詳細安裝與環境需求請參閱 [INSTALL.md](./INSTALL.md)。
- **安全性**：Agent 具備執行指令權限，請於受信任環境執行。
