---
description: How to use NotebookLM as a RAG knowledge base for ArtCraft development
---

# NotebookLM Knowledge Base

## Overview
NotebookLM MCP is configured as an MCP server in Antigravity. It allows the AI assistant to query Google's NotebookLM for zero-hallucination answers based on uploaded documentation.

**Security**: The MCP config is stored at `~/.gemini/antigravity/mcp_config.json` (user home directory), NOT in the repo. Google credentials are stored by the NotebookLM MCP server in the user's home directory. **No credentials are ever stored in the repo.**

## Setup (Already Done)
1. ✅ NotebookLM MCP server added to `~/.gemini/antigravity/mcp_config.json`
2. ✅ Configuration: `npx -y notebooklm-mcp@latest`
3. 🔲 First-time Google authentication (user must do this interactively)

## First-Time Authentication
1. Open Antigravity and start a conversation
2. Say: "Log me in to NotebookLM"
3. A Chrome window will open → log in with your Google account
4. Authentication is saved locally (one-time setup per machine)

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

## Environment Details
- **Work Machine**: Windows 11 PC
- **Home Machine**: Windows 11 PC
- **Laptop**: MacBook (rare use)
- **Build requirements**: Rust 1.88+, Node 24+, CMake, NASM, SQLX_OFFLINE=true
