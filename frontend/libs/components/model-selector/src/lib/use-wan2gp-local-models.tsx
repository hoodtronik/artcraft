import { useState, useEffect } from "react";
import { getWan2gpStatus, getWan2gpModels, type Wan2gpModelInfo } from "@storyteller/tauri-api";
import { VideoModel, ImageModel, ModelCreator, getCreatorIcon } from "@storyteller/model-list";
import { GenerationProvider } from "@storyteller/api-enums";
import type { PopoverItem } from "@storyteller/ui-popover";

export type LocalModelCategory = "video" | "image";

/**
 * Extract the model family from a model name.
 * e.g. "Wan2.1 Alpha v1.0 14B" → "Wan 2.1"
 *      "Wan2.2 Animate 14B" → "Wan 2.2"
 *      "Hunyuan Video 1.5 Upsampler 1080p 8B" → "Hunyuan Video"
 *      "LTX-2 Something" → "LTX"
 */
const FAMILY_PATTERNS: [RegExp, string][] = [
  [/^Wan\s*2\.1\b/i, "Wan 2.1"],
  [/^Wan\s*2\.2\b/i, "Wan 2.2"],
  [/^Hunyuan\s*Video/i, "Hunyuan Video"],
  [/^LTX/i, "LTX Video"],
  [/^CogVideo/i, "CogVideoX"],
  [/^Kandinsky/i, "Kandinsky"],
  [/^Flux/i, "Flux"],
];

function getModelFamily(name: string): string {
  for (const [pattern, family] of FAMILY_PATTERNS) {
    if (pattern.test(name)) return family;
  }
  return "Other";
}

/**
 * Groups PopoverItems by model family (extracted from model name)
 * and inserts disabled section header items between groups.
 */
function groupByFamily(
  items: Omit<PopoverItem, "selected">[],
): Omit<PopoverItem, "selected">[] {
  if (items.length === 0) return items;

  // Group by family
  const groups = new Map<string, Omit<PopoverItem, "selected">[]>();
  for (const item of items) {
    const family = getModelFamily(item.label);
    if (!groups.has(family)) groups.set(family, []);
    groups.get(family)!.push(item);
  }

  // If only one group, skip headers
  if (groups.size <= 1) {
    const all = Array.from(groups.values()).flat();
    all.sort((a, b) => a.label.localeCompare(b.label));
    return all;
  }

  // Sort groups alphabetically
  const sortedKeys = Array.from(groups.keys()).sort((a, b) =>
    a.localeCompare(b),
  );

  // Sort items within each group alphabetically
  for (const groupItems of groups.values()) {
    groupItems.sort((a, b) => a.label.localeCompare(b.label));
  }

  // Flatten with section headers
  const result: Omit<PopoverItem, "selected">[] = [];
  for (let i = 0; i < sortedKeys.length; i++) {
    const family = sortedKeys[i];
    const groupItems = groups.get(family)!;

    // Section header (disabled, non-clickable)
    result.push({
      label: family,
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
              };
            }
          );
        }

        // Group by model family and insert section headers
        const grouped = groupByFamily(modelItems);
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
