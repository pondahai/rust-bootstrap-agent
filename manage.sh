#!/bin/bash

# --- Rust Bootstrap Agent Management Script ---
# Usage: 
#   ./manage.sh setup          - Interactive configuration (LLM & Telegram)
#   ./manage.sh local          - Start Interactive CLI Agent
#   ./manage.sh bot start      - Start Telegram Bridge in background
#   ./manage.sh bot stop       - Stop Telegram Bridge
#   ./manage.sh bot restart    - Restart Telegram Bridge
#   ./manage.sh bot logs       - View Telegram Bridge logs
#   ./manage.sh status         - Check running status
#   ./manage.sh test-evolution - Run automated evolution verification test

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
        read -p "Enter LLM API URL [default: http://192.168.0.110:8001/v1/chat/completions]: " llm_url
        llm_url=${llm_url:-"http://192.168.0.110:8001/v1/chat/completions"}
        read -p "Enter LLM Model Name [default: openai/gpt-oss-120b]: " llm_model
        llm_model=${llm_model:-"openai/gpt-oss-120b"}
        read -p "Enter Telegram Bot Token (leave empty to skip): " tg_token
        
        echo "LLM_API_URL=$llm_url" > "$ROOT_ENV"
        echo "LLM_MODEL_NAME=$llm_model" >> "$ROOT_ENV"
        echo "✅ Root .env created."

        if [ ! -z "$tg_token" ]; then
            echo "TELEGRAM_BOT_TOKEN=$tg_token" > "$BOT_ENV"
            echo "✅ Telegram .env created."
        fi
        echo "--- 🎉 Setup Complete! ---"
        ;;
    "local")
        echo "🚀 Starting Local Interactive CLI Agent..."
        ~/.cargo/bin/cargo build || exit 1
        [ -f "$ROOT_ENV" ] && export $(grep -v '^#' "$ROOT_ENV" | xargs)
        "$EXE_PATH"
        ;;
    "bot")
        case "$2" in
            "start")
                echo "📡 Starting Telegram Bridge..."
                pkill -f "telegram_bridge.py" || true
                sleep 1
                nohup "$VENV_PYTHON" -u "$BRIDGE_PY" > "$BRIDGE_LOG" 2>&1 &
                echo "✅ Telegram Bridge is now running (PID: $!)."
                ;;
            "stop")
                pkill -f "telegram_bridge.py" && echo "✅ Stopped." || echo "⚠️  No bridge running."
                ;;
            "restart")
                echo "🔄 Restarting Telegram Bridge..."
                pkill -f "telegram_bridge.py" || true
                sleep 1
                nohup "$VENV_PYTHON" -u "$BRIDGE_PY" > "$BRIDGE_LOG" 2>&1 &
                echo "✅ Restarted (PID: $!)."
                ;;
            "logs")
                tail -f "$BRIDGE_LOG"
                ;;
            *) echo "Usage: ./manage.sh bot {start|stop|restart|logs}" ;;
        esac
        ;;
    "status")
        echo "--- Current Status ---"
        [ -f "$ROOT_ENV" ] && echo "✅ LLM Config: FOUND" || echo "⚠️  LLM Config: MISSING"
        [ -f "$BOT_ENV" ] && echo "✅ Telegram Config: FOUND" || echo "⚠️  Telegram Config: MISSING"
        pgrep -f "telegram_bridge.py" > /dev/null && echo "✅ Telegram Bridge: RUNNING" || echo "❌ Telegram Bridge: NOT RUNNING"
        ;;
    "test-evolution")
        echo "🧪 Running Automated Evolution Test..."
        ~/.cargo/bin/cargo build || exit 1
        python3 tests/test_evolution.py
        ;;
    *)
        echo "Usage: ./manage.sh {setup | local | bot start|stop|restart|logs | status | test-evolution}"
        ;;
esac
