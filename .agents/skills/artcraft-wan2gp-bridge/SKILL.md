---
name: ArtCraft Wan2GP Bridge Integration
description: Integrating open-source local GPU models into ArtCraft via the Wan2GP headless renderer. Covers the bridge plugin architecture, API contract, and ArtCraft provider integration.
---

# ArtCraft ↔ Wan2GP Bridge Integration

## Project Overview

ArtCraft is a **Rust/Tauri desktop app** that currently only supports cloud-based AI providers (Artcraft, Fal, Grok, Midjourney, Sora, WorldLabs). The goal is to add support for **local open-source models** by connecting ArtCraft to **Wan2GP** (a Python/PyTorch GPU inference engine) via a REST API bridge.

The reference integration is **TronikSlate** (`F:\pinokio\api\TronikSlate`), which runs as a Wan2GP plugin and uses `process_tasks_cli()` for headless rendering.

## Architecture

```
ArtCraft (Rust/Tauri) → HTTP → ArtCraft Bridge Plugin (stdlib http.server :7861) → process_tasks_cli() → GPU
```

The bridge runs **inside** Wan2GP's Python process as a plugin, giving it direct in-process access to the headless render function. No model reloading needed.

## Key Locations

### ArtCraft (Rust/Tauri Desktop App)
- **Workspace:** `F:\__PROJECTS\ArtCraft`
- **Provider enum:** `crates/schema/public/enums/src/common/generation_provider.rs`
  - Currently: `Artcraft, Fal, Grok, Midjourney, Sora, WorldLabs`
  - Needs: `Wan2GP` variant added
- **Video generation command:** `crates/desktop/artcraft/src/core/commands/enqueue/image_to_video/enqueue_image_to_video_command.rs`
  - Has `handle_request()` dispatcher that routes to provider-specific handlers
  - Needs: `handle_wan2gp_video()` handler
- **Frontend:** `frontend/apps/artcraft/app/`
- **API clients:** `crates/api_clients/` (each provider has its own crate)
  - Needs: `wan2gp_client` crate

### Wan2GP (Python/PyTorch Inference Engine)
- **Location:** `F:\pinokio\api\wan.git\app`
- **Core headless render:** `wgp.py` line ~7343 → `process_tasks_cli(queue, state)`
- **Default settings:** `models/_settings.json`
- **Model definitions:** `defaults/*.json` and `finetunes/*.json`
- **Model checkpoints:** `ckpts/`
- **Plugin system:** `shared/utils/plugins.py` → `WAN2GPPlugin` base class

### ArtCraft Bridge Plugin (OUR CODE)
- **Location:** `F:\pinokio\api\wan.git\app\plugins\artcraft-bridge\`
- **Main file:** `plugin.py` — stdlib HTTP API server + task execution engine
- **Config:** Plugin is enabled in `wgp_config.json` → `enabled_plugins` list
- **API Port:** 7861 (configurable via `ARTCRAFT_BRIDGE_PORT` env var)

### TronikSlate (Reference Integration)
- **Location:** `F:\pinokio\api\TronikSlate`
- **Render engine:** `render_engine.py` — shows how to build tasks and call `process_tasks_cli`
- **Model scanner:** `model_scanner.py` — shows how to discover installed models
- **Plugin:** `plugin.py` — full Wan2GP plugin integration reference

## API Contract (Bridge Plugin → ArtCraft)

Base URL: `http://localhost:7861`

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/artcraft/api/status` | Server status, GPU info |
| `GET` | `/artcraft/api/models` | Installed video + image models |
| `GET` | `/artcraft/api/models/profiles/<id>` | Speed profiles for a model |
| `POST` | `/artcraft/api/generate` | Submit generation task (async) |
| `GET` | `/artcraft/api/tasks/<id>` | Poll task status/progress |
| `GET` | `/artcraft/api/tasks/<id>/download` | Download result file |
| `POST` | `/artcraft/api/tasks/<id>/cancel` | Cancel task |

### POST /artcraft/api/generate — Request Body
```json
{
  "model": "i2v_2_2",
  "prompt": "A cat walking...",
  "output_type": "video",
  "resolution": "832x480",
  "seed": -1,
  "num_inference_steps": 30,
  "guidance_scale": 5,
  "video_length": 81,
  "image_start": "<base64>",
  "image_end": "<base64>",
  "image_refs": ["<base64>"],
  "profile_params": {},
  "extra_params": {}
}
```

### Task Status Response
```json
{
  "id": "a1b2c3d4",
  "status": "pending|running|complete|failed|cancelled",
  "progress": 0.5,
  "progress_message": "Rendering...",
  "result_path": "/path/to/output.mp4",
  "download_url": "/artcraft/api/tasks/a1b2c3d4/download",
  "error": null
}
```

## Implementation Status

### ✅ Phase 1: Bridge Plugin (Complete)
- Wan2GP plugin with Python stdlib HTTP server (zero extra dependencies)
- Model discovery (scans defaults/ and finetunes/ for installed models)
- Profile scanning (speed accelerator loras)
- Task queue with background worker thread
- TronikSlate-style task building (`_build_cli_state`, `_build_task`)
- Base64 image input handling
- Output file detection
- CORS headers for Tauri app
- Test script

### 🟡 Phase 2: ArtCraft Rust Integration (In Progress)
1. ✅ Add `Wan2gp` to `GenerationProvider` enum (+ all match arms, tests)
2. ✅ Add `Wan2gp` to `GenerationServiceProvider` (frontend events)
3. ✅ Create `wan2gp_client` crate in `crates/api_clients/` (reqwest HTTP client)
4. 🔲 Add Wan2GP handler in `image_to_video` enqueue command
5. 🔲 Wire up `Wan2gp` in `handle_request()` dispatcher
6. 🔲 Add wan2gp_client dependency to desktop app crate

#### ArtCraft Rust Architecture Notes
- Each provider has its own handler module under `image_to_video/{provider}/`
- Handlers return `Result<TaskEnqueueSuccess, GenerateError>`
- `TaskEnqueueSuccess` records task type, model, provider, and provider_job_id
- The `enqueue_image_to_video_command.rs` dispatcher selects provider based on model + request
- `GenerationProvider` enum has 16-char max serialized length (for MySQL/sqlite)
- Use `reqwest.workspace = true` for HTTP (not wreq — that's for anti-fingerprinting)

### 🔲 Phase 3: ArtCraft Frontend (Not Started)
1. Add "Local (Wan2GP)" provider option in model picker
2. Fetch and display models from bridge API
3. Settings page for bridge URL configuration
4. Progress display from task polling

## Critical Rules

> **DO NOT modify any Wan2GP core files** (`wgp.py`, `shared/`, etc.)
> Only modify files in:
> - `F:\pinokio\api\wan.git\app\plugins\artcraft-bridge\` (our plugin)
> - `F:\__PROJECTS\ArtCraft\` (the ArtCraft app)
>
> The only wan2gp config change allowed was adding `"artcraft-bridge"` to the `enabled_plugins` list in `wgp_config.json`.

## Known Issues & Gotchas

1. **Gradio version:** The Wan2GP Gradio version doesn't support `gr.Code(language="text")`. Use `gr.Markdown` with fenced code blocks instead.
2. **No Flask/FastAPI:** Wan2GP's Python env does NOT have Flask or FastAPI installed. Use Python's built-in `http.server` module instead. Zero extra dependencies.
3. **Plugin directory naming:** Wan2GP plugins use dashes in directory names (e.g., `artcraft-bridge`). This works because `importlib.import_module()` handles it, but the `__init__.py` must be empty (not importing from submodules).
4. **Plugin UI construction:** `create_ui()` is called during Gradio's tab construction. Rules:
   - Do NOT wrap content in `gr.Blocks()` — the parent tab context handles this
   - Do NOT start servers/threads directly inside `create_ui()` — use `threading.Timer(3.0, ...)` to delay
   - Always wrap server starts in try/except so they can NEVER crash Wan2GP
   - Use `_start_api_server_safe()` wrapper pattern
5. **Model loading:** Wan2GP needs models pre-loaded. The bridge assumes the user has started Wan2GP and loaded a model before making requests.
6. **Single GPU:** Only one generation can run at a time. The bridge queues tasks sequentially.
7. **`wgp_config.json`** is the ONLY wan2gp file we touched — to add our plugin to `enabled_plugins`.
8. **Python env:** Wan2GP's python is at `F:\pinokio\api\wan.git\app\env\Scripts\python.exe`

