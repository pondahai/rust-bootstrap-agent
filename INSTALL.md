# 首次安裝與快速設定指南 (Installation Guide)

本文件引導您從零開始部署 **Rust Bootstrap Agent**。

## 🛠️ 第一步：環境準備 (Prerequisites)

確保您的 Linux/WSL 系統已安裝以下工具：
- **Rust**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Python 3.12+**: `sudo apt install python3-venv python3-pip`
- **Git**: 用於管理與同步代碼。

## 📦 第二步：自動化初始化 (Initialization)

請執行以下指令，一鍵完成編譯與環境建置：

```bash
# 1. 給予管理腳本權限
chmod +x manage.sh

# 2. 編譯 Rust 核心 (大腦)
cargo build

# 3. 建立 Python 虛擬環境 (橋接器)
python3 -m venv plugins/venv
source plugins/venv/bin/activate
pip install python-telegram-bot==20.3 python-dotenv
deactivate
```

## 🔑 第三步：金鑰與 API 配置 (Configuration)

### 1. Telegram Bot Token
在 `plugins/` 目錄下建立 `.env` 檔案，並填入由 [@BotFather](https://t.me/botfather) 核發的 Token：
```bash
echo "TELEGRAM_BOT_TOKEN=您的_BOT_TOKEN_在這邊" > plugins/.env
```

### 2. LLM API 配置
開啟 `src/main.rs`，修改以下常數以對接您的 LLM 服務器：
```rust
const API_ENDPOINT: &str = "http://您的_IP_地址:8001/v1/chat/completions";
const MODEL_NAME: &str = "openai/您的_模型_名稱";
```
*修改後請執行 `cargo build` 重新編譯。*

---

## 🚀 啟動對話 (Get Started)

現在您可以使用全能腳本開啟不同介面：

- **本地啟動**: `./manage.sh local` (互動式 CLI，具備完整追蹤)
- **Telegram 啟動**: `./manage.sh bot start` (背景運行，支援手機遙控)

## 🔍 常見問題與除錯 (FAQ)

### 1. 為什麼 Telegram 沒反應？
- 檢查日誌：`./manage.sh bot logs`。
- 確認網路：執行 `curl -v https://api.telegram.org` 測試通連性。
- 如果需要代理：在 `plugins/.env` 加入 `HTTPS_PROXY=http://127.0.0.1:7890`。

### 2. 歷史紀錄在哪裡？
- 所有的對話記憶都儲存在 `storage/history.md` (JSON 格式)。
- 如果想「失憶」，請執行 `./manage.sh local` 後輸入 `/clear`。

---
*文件建立於 2026-03-17*
