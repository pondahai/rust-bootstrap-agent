import os
import subprocess
import asyncio
import logging
import concurrent.futures
from telegram import Bot
from telegram.error import NetworkError, Conflict
from dotenv import load_dotenv

# Setup logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

# Load .env
env_path = os.path.join(os.path.dirname(__file__), '.env')
load_dotenv(dotenv_path=env_path)

TOKEN = os.getenv('TELEGRAM_BOT_TOKEN')
if not TOKEN:
    raise RuntimeError('TELEGRAM_BOT_TOKEN not set')

def run_agent_sync(user_input: str) -> str:
    """Execute the Rust Agent using command-line arguments (Persistent Memory)."""
    project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
    exe_path = os.path.join(project_root, 'target', 'debug', 'rust-bootstrap-agent')
    
    if not os.path.isfile(exe_path):
        exe_path = os.path.join(project_root, 'rust-bootstrap-agent')
            
    try:
        # Now we just pass the input as an argument
        # Rust will load history.md, process it, and save back automatically
        result = subprocess.run([exe_path, user_input], capture_output=True, text=True, timeout=300)
        
        # In single-input mode, Rust prints only the new assistant message to stdout
        reply = result.stdout.strip()
        if not reply and result.stderr:
            return f"❌ [Agent Error] {result.stderr.strip()}"
        return reply or "🤖 (Agent processed but returned no text)"
        
    except Exception as e:
        logger.error(f"Agent Execution Error: {e}")
        return f'❌ [Bridge Error] {str(e)}'

async def main():
    bot = Bot(TOKEN)
    offset = 0
    # Clear backlog
    try:
        updates = await bot.get_updates(offset=-1, timeout=5)
        if updates: offset = updates[0].update_id + 1
    except: pass

    print("--- 🧠 TELEGRAM BRIDGE READY (WITH MEMORY) ---")
    print("Listening for messages... (Async Mode)")

    executor = concurrent.futures.ThreadPoolExecutor(max_workers=3)

    while True:
        try:
            updates = await bot.get_updates(offset=offset, timeout=10)
            for update in updates:
                offset = update.update_id + 1
                if update.message and update.message.text:
                    user_msg = update.message.text
                    chat_id = update.message.chat_id
                    
                    logger.info(f"Telegram ({chat_id}): {user_msg}")
                    
                    # Run agent in thread to avoid blocking polling
                    loop = asyncio.get_running_loop()
                    reply = await loop.run_in_executor(executor, run_agent_sync, user_msg)
                    
                    await bot.send_message(chat_id=chat_id, text=reply)
                    logger.info(f"Replied to {chat_id}")
                    
        except Conflict:
            logger.error("Conflict detected! Retrying in 5s...")
            await asyncio.sleep(5)
        except Exception as e:
            # logger.error(f"Loop Error: {e}")
            await asyncio.sleep(1)

if __name__ == '__main__':
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
