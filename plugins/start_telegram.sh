#!/bin/bash

# Rust Bootstrap Agent - Telegram Bridge Startup Script
# This script ensures the Telegram Bridge runs correctly and communicates with the Rust Agent.

# 1. Path Setup
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENV_PYTHON="$SCRIPT_DIR/venv/bin/python"
BRIDGE_PY="$SCRIPT_DIR/telegram_bridge.py"

echo "--- 🚀 Telegram Bridge Startup ---"

# 2. Pre-flight Checks
if [ ! -f "$VENV_PYTHON" ]; then
    echo "❌ Error: Python venv not found at $VENV_PYTHON"
    exit 1
fi

if [ ! -f "$BRIDGE_PY" ]; then
    echo "❌ Error: Bridge script not found at $BRIDGE_PY"
    exit 1
fi

# 3. Clean up existing instances to prevent 'Conflict'
echo "🧹 Cleaning up existing instances..."
pkill -f "telegram_bridge.py" || true
sleep 1

# 4. Start the Bridge
echo "📡 Starting Telegram Bridge..."
echo "--------------------------------------------------"
echo "Bot is now listening. Press Ctrl+C to stop."
echo "--------------------------------------------------"

# Run in foreground so you can see logs and use Ctrl+C
"$VENV_PYTHON" -u "$BRIDGE_PY"
