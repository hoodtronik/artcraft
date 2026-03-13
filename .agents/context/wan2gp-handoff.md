# Wan2GP Integration — Agent Handoff Notes

> **Last updated:** 2026-03-13 15:55 ET
> **Branch:** `feature/wan2gp-integration`
> **Skill:** `.agents/skills/artcraft-wan2gp-bridge/SKILL.md` — READ THIS FIRST

## What This Project Is

ArtCraft is a Tauri desktop app (Rust backend + React/TS frontend) for AI media generation.
We're adding support for **local GPU models** via Wan2GP, a Python/PyTorch inference engine.
A bridge plugin runs inside Wan2GP and exposes an HTTP API on port 7861.

## What's Been Done (This Session — 2026-03-13)

### 1. Cloud/Local Toggle on All Pages
- Extended `useWan2gpLocalModels` hook with `category` param (`"video"` | `"image"`)
- Added `localModelCategory` prop to `ClassyModelSelector`
- Enabled toggle on: **Video** (`ImageToVideo.tsx`), **Image** (`TextToImage.tsx`), **Edit** (`PageEdit.tsx`)

### 2. Text-to-Image Wan2GP Pipeline (Full Stack)
- **Frontend:** `EnqueueTextToImage.ts` detects `wan2gp_` prefix → sends `wan2gp_local` + `wan2gp_model_id` + `provider: "wan2gp"`
- **Backend:** Added `Wan2gpLocal` to `TextToImageModel` enum, threaded `Wan2gpSettings` through full command chain
- **Handler:** Created `handle_wan2gp_image.rs` under `text_to_image/wan2gp/`
- **Gotchas resolved:** Different `CommonAspectRatio` import paths (video vs image), exhaustive match arms

### 3. Cancel/Stop Button
- Created `wan2gp_cancel_task_command` (Rust Tauri command)
- Added `cancelWan2gpTask` frontend API function
- Red "Stop" button on `InProgressCard` for local tasks
- "Local GPU" subtitle label on Wan2GP tasks in TaskQueue

### 4. Resolution Mapping
- Maps `CommonAspectRatio` → concrete WxH strings for both video and image handlers
- Video: uses `artcraft_router::api::common_aspect_ratio::CommonAspectRatio`
- Image: uses `crate::core::api_adapters::aspect_ratio::common_aspect_ratio::CommonAspectRatio`

## What Needs To Be Done Next

See `wan2gp-tasks.md` for the full prioritized task list. Top items:

1. **Image Edit Wan2GP Pipeline** — toggle is on page, backend handler missing
   - Follow checklist in SKILL.md → "Adding Wan2GP Support to a New Page"
   - Edit models: Flux 2 Klein, Flux 2 Dev, Qwen edit

2. **Result File Download + Display** — polling thread detects completion but doesn't download output

3. **Dynamic Resolution Matching** — user has custom presets (1920×1088, etc.), need to query bridge

## Key Gotchas for Next Agent

1. **CommonAspectRatio comes from different crates** depending on the page type. Wrong import = 14 type errors.
2. **Adding a model variant** requires updating 3+ exhaustive matches (mapping, artcraft handler, provider routing).
3. **Frontend provider override** — when setting `provider: "wan2gp"`, guard against later `if (request.provider)` overwriting it.
4. **Build env** — needs `SQLX_OFFLINE=true`, `LIBCLANG_PATH`, NASM in PATH. See SKILL.md for exact commands.
5. **Rust toolchain pinned to 1.88.0** — sqlx breaks on 1.93+.

## NotebookLM RAG
- Workflow: `.agents/workflows/notebooklm-rag.md`
- Has ArtCraft architecture docs, bridge plugin source, Wan2GP docs uploaded
- Useful for researching ArtCraft patterns if you need context beyond these notes

## Files Modified This Session
```
# Frontend
frontend/libs/components/model-selector/src/lib/use-wan2gp-local-models.tsx
frontend/libs/components/model-selector/src/lib/classy-model-selector.tsx
frontend/apps/artcraft/app/src/pages/PageImage/TextToImage.tsx
frontend/apps/artcraft/app/src/pages/PageEdit/PageEdit.tsx
frontend/apps/artcraft/app/src/pages/PageVideo/ImageToVideo.tsx
frontend/libs/tauri-api/src/lib/wan2gp/Wan2gpApi.ts
frontend/libs/tauri-api/src/lib/enqueue/EnqueueTextToImage.ts
frontend/apps/artcraft/app/src/components/signaled/TopBar/TaskQueue.tsx

# Backend (Rust)
crates/desktop/artcraft/src/services/wan2gp/commands/wan2gp_cancel_task_command.rs  (NEW)
crates/desktop/artcraft/src/services/wan2gp/commands/mod.rs
crates/desktop/artcraft/src/lib.rs
crates/desktop/artcraft/src/core/commands/enqueue/text_to_image/wan2gp/mod.rs  (NEW)
crates/desktop/artcraft/src/core/commands/enqueue/text_to_image/wan2gp/handle_wan2gp_image.rs  (NEW)
crates/desktop/artcraft/src/core/commands/enqueue/text_to_image/enqueue_text_to_image_command.rs
crates/desktop/artcraft/src/core/commands/enqueue/text_to_image/text_to_image_models.rs
crates/desktop/artcraft/src/core/commands/enqueue/text_to_image/artcraft/handle_text_to_image_artcraft.rs
crates/desktop/artcraft/src/core/commands/enqueue/text_to_image/mod.rs
crates/desktop/artcraft/src/core/commands/enqueue/image_to_video/wan2gp/handle_wan2gp_video.rs
```
