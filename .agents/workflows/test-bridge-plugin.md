---
description: How to test the ArtCraft Bridge plugin running inside Wan2GP
---

# Testing the ArtCraft Bridge Plugin

## Prerequisites
- Wan2GP must be running with the `artcraft-bridge` plugin enabled
- The plugin is at `F:\pinokio\api\wan.git\app\plugins\artcraft-bridge\`
- It was enabled by adding `"artcraft-bridge"` to `wgp_config.json` → `enabled_plugins`

## Starting Wan2GP

// turbo
1. Start Wan2GP through Pinokio, or manually:
```
cd F:\pinokio\api\wan.git\app
conda_hook & conda deactivate & conda deactivate & conda deactivate & conda activate base & F:\pinokio\api\wan.git\app\env\Scripts\activate F:\pinokio\api\wan.git\app\env && python wgp.py --multiple-images
```

2. Look for this line in the output confirming the plugin loaded:
```
Loaded plugin: ArtCraft Bridge (from artcraft-bridge)
```

3. Look for:
```
[ArtCraft Bridge] API server starting on port 7861
```

## Quick Smoke Test

// turbo
4. Check if the API is responding:
```powershell
Invoke-RestMethod -Uri "http://localhost:7861/artcraft/api/status"
```

// turbo
5. List available models:
```powershell
Invoke-RestMethod -Uri "http://localhost:7861/artcraft/api/models" | ConvertTo-Json -Depth 5
```

## Full Test Suite

// turbo
6. Run the test script (without generation):
```powershell
cd F:\pinokio\api\wan.git\app\plugins\artcraft-bridge
python test_bridge.py
```

7. Run the test script WITH a test generation (will use GPU):
```powershell
cd F:\pinokio\api\wan.git\app\plugins\artcraft-bridge
python test_bridge.py --generate
```

## Troubleshooting

- **Plugin not loading:** Check `wgp_config.json` has `"artcraft-bridge"` in `enabled_plugins`
- **Crash on startup:** Check the error — likely a Gradio API incompatibility in `plugin.py`'s `create_ui()` method
- **Port conflict:** Set `ARTCRAFT_BRIDGE_PORT` environment variable to a different port
- **No models found:** Ensure model checkpoints exist in `F:\pinokio\api\wan.git\app\ckpts\`

## IMPORTANT
> **Never modify Wan2GP core files** (wgp.py, shared/, etc.)
> Only modify files in `plugins/artcraft-bridge/` and `F:\__PROJECTS\ArtCraft\`
