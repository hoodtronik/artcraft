import { useState, useEffect } from "react";
import { getWan2gpStatus, getWan2gpModels, type Wan2gpModelInfo } from "@storyteller/tauri-api";
import { VideoModel, ImageModel, ModelCreator, getCreatorIcon } from "@storyteller/model-list";
import { GenerationProvider } from "@storyteller/api-enums";
import type { PopoverItem } from "@storyteller/ui-popover";

export type LocalModelCategory = "video" | "image";

/** Pretty-print architecture names for section headers */
const ARCH_DISPLAY_NAMES: Record<string, string> = {
  "wan2.1": "Wan 2.1",
  "wan2.2": "Wan 2.2",
  "ltx_video": "LTX Video",
  "ltx-video": "LTX Video",
  "ltx_video_0.9.7": "LTX Video",
  "hunyuan_video": "Hunyuan Video",
  "hunyuan-video": "Hunyuan Video",
  "cogvideox": "CogVideoX",
  "kandinsky": "Kandinsky",
  "flux": "Flux",
  "wan": "Wan",
};

function getArchDisplayName(arch: string): string {
  const lower = arch.toLowerCase();
  if (ARCH_DISPLAY_NAMES[lower]) return ARCH_DISPLAY_NAMES[lower];
  // Fallback: capitalize first letter
  return arch.charAt(0).toUpperCase() + arch.slice(1);
}

/**
 * Groups PopoverItems by architecture (extracted from description "arch · ...")
 * and inserts disabled section header items between groups.
 */
function groupByArchitecture(
  items: (Omit<PopoverItem, "selected"> & { _arch?: string })[],
): Omit<PopoverItem, "selected">[] {
  if (items.length === 0) return items;

  // Group by arch
  const groups = new Map<string, Omit<PopoverItem, "selected">[]>();
  for (const item of items) {
    const arch = item._arch || "Other";
    if (!groups.has(arch)) groups.set(arch, []);
    groups.get(arch)!.push(item);
  }

  // Sort groups alphabetically by display name
  const sortedKeys = Array.from(groups.keys()).sort((a, b) =>
    getArchDisplayName(a).localeCompare(getArchDisplayName(b)),
  );

  // Sort items within each group alphabetically
  for (const items of groups.values()) {
    items.sort((a, b) => a.label.localeCompare(b.label));
  }

  // Flatten with section headers
  const result: Omit<PopoverItem, "selected">[] = [];
  for (let i = 0; i < sortedKeys.length; i++) {
    const arch = sortedKeys[i];
    const groupItems = groups.get(arch)!;
    const displayName = getArchDisplayName(arch);

    // Section header (disabled, non-clickable)
    result.push({
      label: `── ${displayName} ──`,
      disabled: true,
      divider: i > 0,
    });

    result.push(...groupItems);
  }

  return result;
}

/**
 * Hook that fetches available models from the Wan2GP bridge API.
 * Returns them as PopoverItem[] ready for the ClassyModelSelector.
 *
 * @param category - Which type of models to fetch: "video" (default) or "image"
 */
export function useWan2gpLocalModels(category: LocalModelCategory = "video"): {
  models: Omit<PopoverItem, "selected">[];
  isLoading: boolean;
  isOnline: boolean;
  error: string | null;
} {
  const [models, setModels] = useState<Omit<PopoverItem, "selected">[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isOnline, setIsOnline] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function fetchModels() {
      setIsLoading(true);
      setError(null);

      try {
        // Check bridge status first
        const status = await getWan2gpStatus();
        if (!status.online) {
          setIsOnline(false);
          setModels([]);
          setError("Wan2GP bridge is offline. Start Wan2GP with the ArtCraft Bridge plugin.");
          setIsLoading(false);
          return;
        }

        setIsOnline(true);

        // Fetch models from the bridge
        const modelsResp = await getWan2gpModels();
        if (cancelled) return;

        let modelItems: Omit<PopoverItem, "selected">[];

        if (category === "image") {
          // Map image models
          modelItems = (modelsResp.image || []).map(
            (m: Wan2gpModelInfo) => {
              const model = new ImageModel({
                id: `wan2gp_${m.id}`,
                tauriId: `wan2gp_${m.id}`,
                fullName: m.name,
                category: "image",
                creator: ModelCreator.Wan2GP,
                selectorName: m.name,
                selectorDescription: `${m.architecture} · Local GPU`,
                selectorBadges: ["local"],
                providers: [GenerationProvider.Wan2gp],
                maxGenerationCount: 4,
                defaultGenerationCount: 1,
                canUseImagePrompt: true,
                maxImagePromptCount: 6,
                canChangeAspectRatio: true,
                canTextToImage: true,
                progressBarTime: 60000, // Local models are slower; 60s default
              });

              return {
                label: m.name,
                icon: getCreatorIcon(ModelCreator.Wan2GP),
                description: `${m.architecture} · Local GPU`,
                model: model,
                modelConfig: model.toLegacyModelConfig(),
                _arch: m.architecture,
              };
            }
          );
        } else {
          // Map video models (default)
          modelItems = (modelsResp.video || []).map(
            (m: Wan2gpModelInfo) => {
              const model = new VideoModel({
                id: `wan2gp_${m.id}`,
                tauriId: `wan2gp_${m.id}`,
                fullName: m.name,
                category: "video",
                creator: ModelCreator.Wan2GP,
                selectorName: m.name,
                selectorDescription: `${m.architecture} · ${m.is_i2v ? "Image→Video" : "Text→Video"}`,
                selectorBadges: [m.is_i2v ? "i2v" : "t2v"],
                startFrame: m.is_i2v,
                endFrame: false,
                requiresImage: m.is_i2v,
                providers: [GenerationProvider.Wan2gp],
                supportsReferenceMode: false,
              });

              return {
                label: m.name,
                icon: getCreatorIcon(ModelCreator.Wan2GP),
                description: `${m.architecture} · Local GPU`,
                model: model,
                modelConfig: model.toLegacyModelConfig(),
                _arch: m.architecture,
              };
            }
          );
        }

        // Group by architecture and insert section headers
        const grouped = groupByArchitecture(modelItems);
        setModels(grouped);
      } catch (e: any) {
        if (!cancelled) {
          setError(e?.message || "Failed to fetch Wan2GP models");
          setModels([]);
        }
      } finally {
        if (!cancelled) setIsLoading(false);
      }
    }

    fetchModels();

    // Refresh every 30 seconds while mounted
    const interval = setInterval(fetchModels, 30000);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [category]);

  return { models, isLoading, isOnline, error };
}
