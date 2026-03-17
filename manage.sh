#!/bin/bash

# --- Rust Bootstrap Agent Management Script ---
# Usage: 
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

case "$1" in
    "local")
        echo "🚀 Starting Local Interactive CLI Agent..."
        # Ensure latest build
        ~/.cargo/bin/cargo build || exit 1
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
                echo "Logs are being written to $BRIDGE_LOG"
                ;;
            "stop")
                echo "🛑 Stopping Telegram Bridge..."
                pkill -f "telegram_bridge.py" && echo "✅ Stopped." || echo "⚠️  No bridge running."
                ;;
            "logs")
                echo "📋 Showing last 20 lines of logs (Ctrl+C to exit):"
                tail -f "$BRIDGE_LOG"
                ;;
            *)
                echo "Usage: ./manage.sh bot {start|stop|logs}"
                ;;
        esac
        ;;
    "status")
        echo "--- Current Status ---"
        if pgrep -f "telegram_bridge.py" > /dev/null; then
            echo "✅ Telegram Bridge: RUNNING (PID: $(pgrep -f 'telegram_bridge.py'))"
        else
            echo "❌ Telegram Bridge: NOT RUNNING"
        fi
        [ -f "$EXE_PATH" ] && echo "✅ Rust Agent Binary: READY" || echo "❌ Rust Agent Binary: MISSING (Run cargo build)"
        ;;
    *)
        echo "Usage: ./manage.sh {local | bot start|stop|logs | status}"
        echo "  - local:     Start interactive CLI"
        echo "  - bot start: Run Telegram Bridge in background"
        echo "  - bot stop:  Stop Telegram Bridge"
        echo "  - bot logs:  View Telegram logs"
        echo "  - status:    Check everything"
        ;;
esac
