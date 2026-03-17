#!/bin/bash

# --- Rust Bootstrap Agent Management Script ---
# Usage: 
#   ./manage.sh setup      - Interactive configuration (LLM & Telegram)
#   ./manage.sh local      - Start Interactive CLI Agent
#   ./manage.sh bot start  - Start Telegram Bridge in background
#   ./manage.sh bot stop   - Stop Telegram Bridge
#   ./manage.sh bot logs   - View Telegram Bridge logs
#   ./manage.sh status     - Check running status

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXE_PATH="$PROJECT_ROOT/target/debug/rust-bootstrap-agent"
VENV_PYTHON="$PROJECT_ROOT/plugins/venv/bin/python"
BRIDGE_PY="$PROJECT_ROOT/plugins/telegram_bridge.py"
BRIDGE_LOG="$PROJECT_ROOT/plugins/bridge.log"
ROOT_ENV="$PROJECT_ROOT/.env"
BOT_ENV="$PROJECT_ROOT/plugins/.env"

# Load root .env if exists for local session
[ -f "$ROOT_ENV" ] && export $(grep -v '^#' "$ROOT_ENV" | xargs)

case "$1" in
    "setup")
        echo "--- 🛠️  Interactive Setup ---"
        
        # 1. LLM Configuration
        read -p "Enter LLM API URL [default: http://192.168.0.110:8001/v1/chat/completions]: " llm_url
        llm_url=${llm_url:-"http://192.168.0.110:8001/v1/chat/completions"}
        
        read -p "Enter LLM Model Name [default: openai/gpt-oss-120b]: " llm_model
        llm_model=${llm_model:-"openai/gpt-oss-120b"}
        
        # 2. Telegram Configuration
        read -p "Enter Telegram Bot Token (leave empty to skip): " tg_token
        
        # 3. Write Root .env
        echo "LLM_API_URL=$llm_url" > "$ROOT_ENV"
        echo "LLM_MODEL_NAME=$llm_model" >> "$ROOT_ENV"
        echo "✅ Root .env created."

        # 4. Write Bot .env
        if [ ! -z "$tg_token" ]; then
            echo "TELEGRAM_BOT_TOKEN=$tg_token" > "$BOT_ENV"
            echo "HTTPS_PROXY=" >> "$BOT_ENV"
            echo "✅ Telegram .env created."
        fi

        echo "--- 🎉 Setup Complete! ---"
        echo "Now you can run './manage.sh local' or './manage.sh bot start'."
        ;;
    "local")
        echo "🚀 Starting Local Interactive CLI Agent..."
        # Load env into current shell
        if [ -f "$ROOT_ENV" ]; then
            export $(grep -v '^#' "$ROOT_ENV" | xargs)
        fi
        "$EXE_PATH"
        ;;
    "bot")
        case "$2" in
            "start")
                echo "📡 Starting Telegram Bridge in background..."
                pkill -f "telegram_bridge.py" || true
                sleep 1
                nohup "$VENV_PYTHON" -u "$BRIDGE_PY" > "$BRIDGE_LOG" 2>&1 &
                echo "✅ Telegram Bridge is now running (PID: $!)."
                ;;
            "stop")
                echo "🛑 Stopping Telegram Bridge..."
                pkill -f "telegram_bridge.py" && echo "✅ Stopped." || echo "⚠️  No bridge running."
                ;;
            "logs")
                tail -f "$BRIDGE_LOG"
                ;;
            *)
                echo "Usage: ./manage.sh bot {start|stop|logs}"
                ;;
        esac
        ;;
    "status")
        echo "--- Current Status ---"
        # Check LLM Config
        if [ -f "$ROOT_ENV" ]; then
            echo "✅ LLM Config: FOUND"
            grep "LLM_API_URL" "$ROOT_ENV"
        else
            echo "⚠️  LLM Config: NOT FOUND (Run ./manage.sh setup)"
        fi
        
        # Check TG Config
        if [ -f "$BOT_ENV" ]; then
            echo "✅ Telegram Config: FOUND"
        else
            echo "⚠️  Telegram Config: NOT FOUND"
        fi

        if pgrep -f "telegram_bridge.py" > /dev/null; then
            echo "✅ Telegram Bridge: RUNNING"
        else
            echo "❌ Telegram Bridge: NOT RUNNING"
        fi
        ;;
    *)
        echo "Usage: ./manage.sh {setup | local | bot start|stop|logs | status}"
        ;;
esac
