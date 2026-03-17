import subprocess
import os
import time

def run_test():
    print("--- 🧪 Starting Automated Evolution Test ---")
    
    project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    agent_path = os.path.join(project_root, "target/debug/rust-bootstrap-agent")
    system_md = os.path.join(project_root, "storage/system.md")
    
    # 清除舊技能標記以利測試
    with open(system_md, "r") as f:
        content = f.read()
    if "LogAnalyzer" in content:
        print("Cleaning old test data from system.md...")
        new_content = content.split("## 🧩 已學會的技能")[0] + "## 🧩 已學會的技能\n*(本區塊由系統自動維護)*\n"
        with open(system_md, "w") as f:
            f.write(new_content)

    # 測試指令
    prompt = "There is a log file at data/dummy_logs.txt. It is quite long. Please write a python script to count the unique error messages, execute it to show results, and then promote this script as a permanent skill named 'LogAnalyzer'."
    
    print(f"Sending prompt to Agent...")
    start_time = time.time()
    
    # 執行 Agent (單次模式)
    try:
        result = subprocess.run([agent_path, prompt], capture_output=True, text=True, timeout=180)
        print(f"Agent finished in {time.time() - start_time:.2f}s")
    except subprocess.TimeoutExpired:
        print("❌ Test Failed: Agent timed out.")
        return

    # 1. 檢查輸出是否包含成功訊息
    print("\n--- Agent Output Excerpt ---")
    print(result.stdout[-500:]) 
    
    # 2. 檢查 plugins 目錄
    plugins = os.listdir(os.path.join(project_root, "plugins"))
    has_analyzer = any("analyzer" in p.lower() for p in plugins)
    
    # 3. 檢查 system.md
    with open(system_md, "r") as f:
        new_system_content = f.read()
    
    print("\n--- Verification Results ---")
    if has_analyzer and "LogAnalyzer" in new_system_content:
        print("✅ SUCCESS: Agent evolved! New skill 'LogAnalyzer' is now permanent.")
    else:
        print("❌ FAILED: Agent did not complete the evolution loop.")
        if not has_analyzer: print("   - Missing script in plugins/")
        if "LogAnalyzer" not in new_system_content: print("   - Missing skill entry in system.md")

if __name__ == "__main__":
    run_test()
