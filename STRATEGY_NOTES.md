# 策略筆記：Agent 自舉與工具自生 (Self-Bootstrapping & Tool Augmentation)

> **當前狀態：已實裝 (Implemented in Stage 7)**  
> 透過 `src/executor.rs` 的升級與 `promote_skill` 命令，Agent 現在已具備將臨時工具轉化為永久技能的閉環能力。

## 🎯 核心哲學：從「工人」進化為「工頭」
當 LLM 面對複雜任務、長文本或推理瓶頸時，不應直接處理原始數據，而應優先考慮 **「為自己打造工具」**。這能節省上下文空間並大幅提升執行準確性。

## 🛠️ 三大核心策略 (The Three Pillars)

### 1. 數據過濾 (Data Filtering - "分而治之")
- **實作方式**：JSON Array 批次執行 + 模擬 Stdin 解析。
- **作法**：引導 Agent 先寫一個 Python/AWK 腳本提取關鍵欄位，再讀取過濾後的結果。
- **目標**：防止上下文爆炸。

### 2. 專用計算 (Specialized Tools - "能力補強")
- **實作方式**：`save_script` + `execute_command`。
- **作法**：將精密運算封裝進 Python 腳本執行，獲取確定性結果。
- **目標**：消除 LLM 的不穩定性。

### 3. 診斷與修復 (Diagnostic & Self-Healing)
- **實作方式**：`execute_command` 捕捉 Exit Status 與 STDERR。
- **作法**：根據錯誤回傳修正下一步行動。

## 🚀 自我進化：技能升遷 (Skill Promotion)
**已達成：**
1. **創生 (Create)**：`save_script`。
2. **驗證 (Verify)**：`execute_command`。
3. **升遷 (Promote)**：`promote_skill` (自動移動檔案並更新 `system.md`)。
4. **內化 (Internalize)**：下次啟動自動識別新技能。

---
*紀錄更新於 2026-03-17*
