---
description: How to use NotebookLM as a RAG knowledge base for ArtCraft development
---

# NotebookLM Knowledge Base

## Overview
NotebookLM MCP is configured as an MCP server in Antigravity. It allows the AI assistant to query Google's NotebookLM for zero-hallucination answers based on uploaded documentation.

**Security**: The MCP config is stored at `~/.gemini/antigravity/mcp_config.json` (user home directory), NOT in the repo. Google credentials are stored by the NotebookLM MCP server at `~/.notebooklm-mcp/auth.json`. **No credentials are ever stored in the repo.**

## Setup (Completed 2026-03-13)
1. ✅ Python package installed: `pip install -U notebooklm-mcp-server` (v0.1.15)
2. ✅ MCP config updated: `~/.gemini/antigravity/mcp_config.json` → `notebooklm-mcp.exe`
3. ✅ Google authentication completed via `notebooklm-mcp-auth` (auto mode)
4. ✅ Auth tokens cached at `~/.notebooklm-mcp/auth.json`

## Re-Authentication (if tokens expire)
1. Close all Chrome instances
2. Run: `& "C:\Users\Animation\AppData\Roaming\Python\Python314\Scripts\notebooklm-mcp-auth.exe"`
3. Log in to Google in the Chrome window that opens
4. Restart Antigravity to reconnect

## Creating the ArtCraft Knowledge Notebook
1. Go to [notebooklm.google.com](https://notebooklm.google.com)
2. Create a new notebook called "ArtCraft Development"
3. Upload these sources:
   - The ArtCraft Bridge skill (`SKILL.md`)
   - Bridge plugin source (`plugins/artcraft-bridge/plugin.py`)
   - Wan2GP client crate documentation
   - Any relevant architecture docs
4. Share the notebook: ⚙️ Share → Anyone with link → Copy
5. Tell the assistant: "Add [link] to library tagged 'artcraft, wan2gp, bridge'"

## Usage
Once authenticated and a notebook is created, the AI assistant can:
- Query the notebook for architecture decisions
- Look up API contracts and endpoint details
- Retrieve implementation patterns
- Check configuration requirements
- Create new notebooks, add sources, generate content

## ArtCraft Development Notebook

**Notebook ID:** `fcf0f496-852a-4140-8dae-d6a344e3e3e1`
**URL:** https://notebooklm.google.com/notebook/fcf0f496-852a-4140-8dae-d6a344e3e3e1

### Current Sources (as of 2026-03-13)
1. **ArtCraft Wan2GP Architecture Guide** — full architecture, data flow, file locations
2. **Agent Handoff Notes — 2026-03-13 Session** — what was done, gotchas, modified files
3. **Next Agent Prompt Template** — copy-paste prompt for starting a new agent session

### How to Query
```
mcp_notebooklm_notebook_query(notebook_id="fcf0f496-852a-4140-8dae-d6a344e3e3e1", query="your question")
```

## ⚠️ MANDATORY: Save Your Work Back to NotebookLM

**Every agent session MUST save context back to the notebook before ending.**

This is how we maintain persistent memory across agent sessions. At the end of your session:

1. **Save handoff notes** — summarize what you did, what's left, and any gotchas:
   ```
   mcp_notebooklm_notebook_add_text(
     notebook_id="fcf0f496-852a-4140-8dae-d6a344e3e3e1",
     title="Agent Handoff Notes — YYYY-MM-DD Session",
     text="<your notes>"
   )
   ```

2. **Update the task list** — check off completed items, add new ones:
   - Edit `.agents/context/wan2gp-tasks.md` in the repo
   - Also add an updated copy to NotebookLM

3. **Update the handoff file** — save to `.agents/context/wan2gp-handoff.md` in the repo

4. **Commit and push** — so both the repo files and NotebookLM stay in sync

## Environment Details
- **Work Machine**: Windows 11 PC
- **Home Machine**: Windows 11 PC
- **Laptop**: MacBook (rare use)
- **Build requirements**: Rust 1.88+, Node 24+, CMake, NASM, SQLX_OFFLINE=true
- **NotebookLM MCP**: `notebooklm-mcp-server` v0.1.15 (Python, pip)
- **Executable**: `C:\Users\Animation\AppData\Roaming\Python\Python314\Scripts\notebooklm-mcp.exe`

