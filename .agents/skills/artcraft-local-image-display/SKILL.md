---
name: ArtCraft Local Image Display
description: How ArtCraft displays locally-generated images from Wan2GP (or any local GPU provider). Covers the full data flow from bridge → polling → database → frontend rendering.
---

# ArtCraft Local Image Display

## Overview

ArtCraft was designed for cloud providers (Storyteller, Sora, Midjourney) that return CDN URLs for generated images. Local providers like Wan2GP need to adapt to this pipeline. This guide documents the gotchas and patterns for making local GPU results display correctly.

## The Display Pipeline (5 stages)

```
Bridge API → Polling Thread → SQLite Database → Frontend Store → UI Components
```

### Stage 1: Bridge API Result
- Bridge plugin saves output to `{wan2gp_root}/outputs/`
- Task status endpoint returns `result_path` and `result_type`
- Download endpoint serves the raw file bytes

### Stage 2: Polling Thread (`wan2gp_task_polling_thread.rs`)
Downloads result bytes and creates a displayable URL:
- **Images**: Uses `data:image/png;base64,...` data URLs (no Tauri permissions needed)
- **Videos**: Uses `file:///` URLs

### Stage 3: SQLite Database Update
The `update_successful_task_status_with_metadata` function requires:
- `maybe_primary_media_file_token` — **MUST be Some()**. Use synthetic `MediaFileToken("wan2gp_{task_id}")`. Without this, `get_task_queue_command.rs` uses `.zip()` that drops the entire `completed_item`.
- `maybe_primary_media_file_cdn_url` — The data URL or file URL string
- `maybe_primary_media_file_class` — `TaskMediaFileClass::Image` or `Video`

### Stage 4: Frontend Events
Two separate event systems drive the UI:

1. **`generation-complete-event`** — Generic. Triggers task queue refresh (notification panel)
2. **`text_to_image_generation_complete_event`** — Specific. Carries `GeneratedImage[]` data with `media_token`, `cdn_url`, `maybe_thumbnail_template`. Drives the main page batch system (`TextToImageStore.completeBatch`)

**Both must be emitted** for full functionality.

### Stage 5: UI Rendering
Key components and their URL handling:

| Component | File | How it uses the URL |
|-----------|------|-------------------|
| Lightbox modal | `lightbox-modal.tsx` | `addCorsParam(url)` → `<img src>` |
| Task queue thumbnails | `TaskQueue.tsx` | `serverThumbnail \|\| cachedThumbnail \|\| cdn_url` |
| Main page grid | `TextToImage.tsx` | `batch.images[].cdn_url` from store |

## Common Gotchas

### 1. `addCorsParam()` breaks non-HTTP URLs
**File**: `libs/common/src/lib/utils/thumbnail-utils.ts`

`addCorsParam()` appends `?cors=1` to all URLs. This breaks `data:`, `file:`, and `blob:` URLs. The fix: skip these protocols.

```typescript
if (url.startsWith("data:") || url.startsWith("file:") || url.startsWith("blob:")) return url;
```

### 2. Missing MediaFileToken causes invisible completed items
**File**: `get_task_queue_command.rs` lines 124-127

```rust
let token_and_url = task.on_complete_primary_media_file_token
    .zip(task.on_complete_primary_media_file_cdn_url);
```

If either is `None`, `completed_item` is `None` and the frontend sees a completed task with no image data.

### 3. `completeBatch` must be called for main page
The `TextToImage.tsx` batch system is separate from the task queue. It requires listening to `text_to_image_generation_complete_event` and calling `completeBatch()`.

### 4. Tauri asset protocol needs explicit scope
Using `https://asset.localhost/` requires configuring `assetProtocol` scope in `capabilities/default.json`. Data URLs avoid this issue entirely.

## File Reference

| File | Purpose |
|------|---------|
| `wan2gp_task_polling_thread.rs` | Polls bridge, downloads results, updates DB, emits events |
| `get_task_queue_command.rs` | Reads DB tasks → frontend DTOs (has `.zip()` guard) |
| `TaskQueue.tsx` | Notification dropdown rendering |
| `TextToImage.tsx` / `TextToImageStore.ts` | Main page batch system |
| `lightbox-modal.tsx` | Full-size image detail view |
| `thumbnail-utils.ts` | `addCorsParam()`, `getThumbnailUrl()`, `PLACEHOLDER_IMAGES` |
