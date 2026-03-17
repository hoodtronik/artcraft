# Wan2GP Image Model Parameters Reference

> Extracted from real Wan2GP queue.json — shows how each image model family
> uses `image_refs`, `image_guide`, `video_prompt_type`, and other key params.

## Quick Reference Table

| Model Type | `video_prompt_type` | `image_refs` | `image_guide` | `model_mode` | Key Notes |
|---|---|---|---|---|---|
| `flux2_klein_9b` | `"KI"` | ✅ array | ✅ (optional) | `0` | 4 steps, `embedded_guidance_scale: 1` |
| `flux2_klein_4b` | `"KI"` | ✅ array | ❌ | `0` | 4 steps, `masking_strength: 0.25` |
| `flux2_dev` | `"KI"` | ✅ array | ❌ | `0` | 30 steps, `denoising_strength: 0.5`, `embedded_guidance_scale: 4` |
| `pi_flux2` | `"KIMV"` | ✅ array | ✅ controlnet | `null` | `denoising_strength: 0.5`, `embedded_guidance_scale: 4` |
| `qwen_image_2512_20B` | `""` | ❌ `null` | ❌ | `2` | Text-only, no ref images, `guidance_scale: 4` |
| `qwen_image_edit_plus_20B` | `"KIDV"` | ✅ array | ✅ controlnet | `0` | Edit model, `denoising_strength: 1` |
| `qwen_image_edit_plus2_20B` | `"KIPV"` | ✅ array | ✅ controlnet | `0` | Edit model v2, `denoising_strength: 1` |

## `video_prompt_type` Flags

The `video_prompt_type` string is a combination of single-letter flags:
- **`K`** — Keep/use reference (standard for all ref-image models)
- **`I`** — Image reference mode (image_refs array provided)
- **`M`** — Masking/inpainting mode
- **`D`** — Depth controlnet
- **`P`** — Pose controlnet
- **`V`** — Video/VACE mode

## Critical Params for Image Generation

All image models use `"image_mode": 1` to signal image (not video) output.

### Reference Images
- Passed as `image_refs: ["filename.ext"]` (array of filenames in zip, or base64 via bridge API)
- The bridge API uses field name `image_refs_b64` for base64-encoded images

### ControlNet / Image Guide
- Passed as `image_guide: "filename.ext"` (separate from reference image)
- Used for depth maps, pose maps, etc.
- The bridge API uses field name `image_guide` (currently NOT supported via REST API)

### Denoising Strength
- Controls how much the model changes from the reference image
- `1.0` = full creative freedom (default for generation)
- `0.5` = preserve more of the original (better for style transfer)
- `0.25` = minimal changes (used by Klein 4B for subtle edits)

### Model Mode
- `0` = Standard mode (most models)
- `2` = Text-only mode (Qwen image gen without reference)
- `null` = Model default

## Full JSON Examples

### Flux Klein 9B with Reference + ControlNet
```json
{
  "model_type": "flux2_klein_9b",
  "image_mode": 1,
  "video_prompt_type": "KI",
  "image_refs": ["task_image_refs_0.png"],
  "image_guide": "task_image_guide_0.png",
  "num_inference_steps": 4,
  "embedded_guidance_scale": 1,
  "denoising_strength": 1
}
```

### Flux Dev with Reference Only
```json
{
  "model_type": "flux2_dev",
  "image_mode": 1,
  "video_prompt_type": "KI",
  "image_refs": ["task_image_refs_0.webp"],
  "image_guide": null,
  "num_inference_steps": 30,
  "embedded_guidance_scale": 4,
  "denoising_strength": 0.5
}
```

### Qwen Image Edit Plus with Reference + Depth ControlNet
```json
{
  "model_type": "qwen_image_edit_plus_20B",
  "image_mode": 1,
  "video_prompt_type": "KIDV",
  "image_refs": ["task_image_refs_0.jpg"],
  "image_guide": "task_image_guide_0.png",
  "num_inference_steps": 30,
  "embedded_guidance_scale": 6,
  "denoising_strength": 1,
  "model_mode": 0,
  "sample_solver": "default"
}
```

### Qwen Pure Text-to-Image (No Reference)
```json
{
  "model_type": "qwen_image_2512_20B",
  "image_mode": 1,
  "video_prompt_type": "",
  "image_refs": null,
  "image_guide": null,
  "num_inference_steps": 30,
  "model_mode": 2,
  "sample_solver": "default"
}
```

## Bridge API Notes

The ArtCraft bridge plugin (`plugin.py`) handles base64 images:
- `image_refs_b64`: Array of base64 strings → saved to temp files → set as `image_refs`
- `image_start_b64`: Single base64 string → saved to temp file → set as `image_start`
- `image_guide` is NOT yet supported via the bridge REST API (only via zip)

## Source
Data extracted from user's real Wan2GP queue.json, March 2026.
