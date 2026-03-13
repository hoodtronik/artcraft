# Wan2GP Integration — Remaining Tasks

> **Last updated:** 2026-03-13 15:55 ET
> **Branch:** `feature/wan2gp-integration`
> **Handoff notes:** `.agents/context/wan2gp-handoff.md`
> **Skill file:** `.agents/skills/artcraft-wan2gp-bridge/SKILL.md`

---

## Priority 1: Image Edit Wan2GP Pipeline

The Cloud/Local toggle is already on the Edit page, but the backend has no handler.
Edit models: Flux 2 Klein, Flux 2 Dev, Qwen edit models.

Follow the checklist in SKILL.md → "Adding Wan2GP Support to a New Page":

- [ ] Find `EnqueueImageEditRequest` in Rust and add `wan2gp_model_id: Option<String>` field
- [ ] Find `ImageEditModel` enum and add `Wan2gpLocal` variant with `#[serde(rename = "wan2gp_local")]`
- [ ] Add `Wan2gpLocal` to `image_edit_model_to_model_type` mapping (safe fallback)
- [ ] Add `Wan2gpLocal` to exhaustive match in artcraft image edit handler
- [ ] Add `(ImageEditModel::Wan2gpLocal, _) => GenerationProvider::Wan2gp` to provider routing
- [ ] Thread `Wan2gpSettings` through image edit command → handle_request → dispatch_request
- [ ] Add `GenerationProvider::Wan2gp => { handle_wan2gp_image_edit(...).await }` dispatch branch
- [ ] Create `image_edit/wan2gp/mod.rs` + `handle_wan2gp_image_edit.rs`
- [ ] Update frontend `EnqueueImageEdit.ts` to detect `wan2gp_` prefix and send `wan2gp_local` + `wan2gp_model_id` + `provider: "wan2gp"`
- [ ] Test end-to-end

## Priority 2: Result File Download + Display

Polling thread detects completion but doesn't download the output file.

- [ ] In polling thread: when task status = "complete", call `GET /artcraft/api/tasks/{id}/download`
- [ ] Save downloaded file to ArtCraft temp/media storage (follow existing video result pattern)
- [ ] Create media file token + URL for the downloaded file
- [ ] Mark task as completed in task database with result info
- [ ] Emit frontend event with result (so TaskQueue shows thumbnail)
- [ ] Verify result appears in gallery/output view

## Priority 3: Dynamic Resolution Matching

Currently hardcoded WxH maps. User has custom Wan2GP presets like 1920×1088.

- [ ] Add `/artcraft/api/resolutions` endpoint to bridge plugin (return all available presets)
- [ ] Add `get_resolutions()` to `wan2gp_client` Rust crate
- [ ] Query bridge for resolution presets at startup or on model select
- [ ] Implement "closest match" algorithm: match by aspect ratio first, then by total pixel count
- [ ] Replace hardcoded resolution maps in `handle_wan2gp_video.rs` and `handle_wan2gp_image.rs`

## Priority 4: Speed Profile Auto-Selection

Bridge endpoint exists (`GET /models/profiles/{id}`), just not wired up.

- [ ] Fetch profiles after model selection
- [ ] Auto-select fastest profile (TronikSlate pattern: `_get_fastest_profile()`)
- [ ] Pass `profile_params` to generate request
- [ ] Add profile selector in Wan2GP settings UI (optional)

## Priority 5: LTX Frame Math

LTX models require video frames = 8n+1 (e.g., 9, 17, 25, 33...).

- [ ] Implement `snap_to_8n1(frames: u32) -> u32` utility
- [ ] Detect LTX model architecture (from model ID or metadata)
- [ ] Apply snap before sending `video_length` to bridge

## Optional: Auto-Launch Wan2GP

- [ ] Detect bridge offline when Wan2GP provider selected
- [ ] Spawn `python wgp.py --multiple-images` as subprocess
- [ ] Wait for bridge API to come online (polling with backoff)
- [ ] Manage process lifecycle (start/stop with ArtCraft)
- [ ] Use existing `subprocess_common` crate pattern

## Optional: First-Run Setup Wizard

- [ ] Detect if Wan2GP is installed on first launch
- [ ] Auto-scan common paths (`F:\pinokio\api\wan.git\app`, `%LOCALAPPDATA%\wan2gp`, etc.)
- [ ] File picker for manual path selection
- [ ] Auto-download + install option
