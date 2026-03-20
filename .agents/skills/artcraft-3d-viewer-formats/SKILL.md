---
name: ArtCraft 3D Viewer Formats
description: How ArtCraft loads and displays 3D model files (PLY, SPZ, GLB) in the Viewer3D component and the 3D Editor scene.
---

# ArtCraft 3D Model Format Support

## Overview
ArtCraft supports loading 3D models in multiple formats. The `Viewer3D` component (used in the Image-to-3D World tab and lightbox) and the `scene.ts` 3D Editor have separate loading paths.

## Viewer3D Component
**Path:** `frontend/libs/components/viewer-3d/src/lib/viewer-3d.tsx`

The `Viewer3D` component takes a `modelUrl` prop and detects format by file extension:

| Format | Extension | Loader | Notes |
|--------|-----------|--------|-------|
| Gaussian Splat | `.spz` | `@sparkjsdev/spark` `SplatMesh` | Rendered with rotation fix (z = π) |
| PLY Mesh | `.ply` | Three.js `PLYLoader` | Vertex colors supported; normals auto-computed |
| GLTF/GLB | `.glb`, `.gltf` | Three.js `GLTFLoader` | Default fallback |

### PLY Loading Details
- `PLYLoader` from `three/examples/jsm/loaders/PLYLoader.js`
- Checks `geometry.hasAttribute("color")` for vertex colors
- Uses `MeshStandardMaterial` with `vertexColors: true` when colors exist
- Falls back to neutral gray `#8899aa` material when no vertex colors
- `DoubleSide` rendering enabled for all PLY files

## 3D Editor (Enigma Scene)
**Path:** `frontend/apps/artcraft/app/src/pages/PageEnigma/Editor/scene.ts`

The 3D Editor `loadObject()` method (line ~570) routes by URL extension:
- `.pmd`/`.pmx` → MMD loader
- `.png`/`.jpg`/`.jpeg`/`.mp4` → Image/video plane
- `.spz` → `loadSplatWithPlaceholder()` (uses `SplatMesh` via Tauri CORS bypass)
- Everything else → `loadGlbWithPlaceholder()` (GLB fallback)

### ⚠️ Known Gap: PLY Not Yet Supported in 3D Editor
The 3D Editor `scene.ts` does **not** currently handle `.ply` files. To add support:
1. Add `PLY = "ply"` to `SPLAT_FILE_TYPE` enum in `frontend/apps/artcraft/app/src/enums/FileTypes.ts`
2. Add `.ply` routing in `scene.ts` `loadObject()` method (line ~601)
3. Create a `loadPlyWithPlaceholder()` method similar to `loadSplatWithPlaceholder()`
4. Update `UploadFilesSplat` component to accept `.ply` files in its file picker

### ⚠️ Future: Image-to-3D UI Integration
The Image-to-3D World tab may need to link to an external local app (like ML-Sharp via Pinokio)
rather than spawning subprocesses directly. This is deferred to a future branch.

## Splat Upload Flow
**Upload Modal:** `frontend/apps/artcraft/app/src/components/reusable/UploadModalSplat/UploadModalSplat.tsx`
- Uses `SPLAT_FILE_TYPE` enum to filter accepted files
- Currently only accepts `.spz` files
- Calls `onLocalBytes(buffer, shouldFlip)` to load splat data directly

## Key Files
- [viewer-3d.tsx](file:///F:/__PROJECTS/ArtCraft/frontend/libs/components/viewer-3d/src/lib/viewer-3d.tsx) — Viewer3D component
- [scene.ts](file:///F:/__PROJECTS/ArtCraft/frontend/apps/artcraft/app/src/pages/PageEnigma/Editor/scene.ts) — 3D Editor scene loader
- [FileTypes.ts](file:///F:/__PROJECTS/ArtCraft/frontend/apps/artcraft/app/src/enums/FileTypes.ts) — File type enums (SPLAT_FILE_TYPE)
- [UploadModalSplat.tsx](file:///F:/__PROJECTS/ArtCraft/frontend/apps/artcraft/app/src/components/reusable/UploadModalSplat/UploadModalSplat.tsx) — Splat upload modal
