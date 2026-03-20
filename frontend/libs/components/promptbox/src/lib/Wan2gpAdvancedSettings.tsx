import { useState, useEffect, useRef, useCallback } from "react";
import { faChevronDown, faGear, faDice, faBolt } from "@fortawesome/pro-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { GenerationProvider } from "@storyteller/api-enums";
import {
  getWan2gpSettings,
  updateWan2gpSettings,
  getWan2gpProfiles,
  type Wan2gpSettings,
  type UpdateWan2gpSettingsRequest,
  type Wan2gpProfileInfo,
} from "@storyteller/tauri-api";
import { twMerge } from "tailwind-merge";

/** Default FPS by model architecture prefix */
const guessFps = (modelId?: string): number => {
  if (!modelId) return 16;
  const id = modelId.toLowerCase();
  if (id.includes("ltx")) return 24;
  if (id.includes("hunyuan") || id.includes("hy")) return 24;
  if (id.includes("wan") || id.includes("i2v") || id.includes("t2v")) return 16;
  return 16;
};

interface Wan2gpAdvancedSettingsProps {
  selectedProvider?: GenerationProvider;
  /** Hide video-only fields (e.g. video length) */
  hideVideoFields?: boolean;
  /** Wan2GP model ID for fetching accelerator profiles */
  wan2gpModelId?: string;
}

/**
 * Collapsible "Advanced Settings" panel for Wan2GP local generation.
 * Only renders when selectedProvider === GenerationProvider.Wan2gp.
 */
export const Wan2gpAdvancedSettings = ({
  selectedProvider,
  hideVideoFields,
  wan2gpModelId,
}: Wan2gpAdvancedSettingsProps) => {
  const [isOpen, setIsOpen] = useState(false);
  const [settings, setSettings] = useState<Wan2gpSettings | null>(null);
  const [loaded, setLoaded] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);
  const [contentHeight, setContentHeight] = useState(0);

  // Profiles
  const [profiles, setProfiles] = useState<Wan2gpProfileInfo[]>([]);
  const [selectedProfileName, setSelectedProfileName] = useState<string | null>(null);
  const [profilesLoading, setProfilesLoading] = useState(false);

  // Video length mode: seconds (default) or frames
  const [lengthMode, setLengthMode] = useState<"seconds" | "frames">("seconds");
  const fps = guessFps(wan2gpModelId);
  const DEFAULT_SECONDS = 5;
  const DEFAULT_FRAMES = DEFAULT_SECONDS * fps; // 80 for 16fps, 120 for 24fps

  const isWan2gp = selectedProvider === GenerationProvider.Wan2gp;

  // Load settings when opened and provider is Wan2GP
  useEffect(() => {
    if (isWan2gp && isOpen && !loaded) {
      getWan2gpSettings()
        .then((s) => {
          setSettings(s);
          setLoaded(true);
        })
        .catch((e) => console.error("Failed to load Wan2GP settings:", e));
    }
  }, [isWan2gp, isOpen, loaded]);

  // Load profiles when model changes
  useEffect(() => {
    if (isWan2gp && isOpen && wan2gpModelId) {
      setProfilesLoading(true);
      // Strip the "wan2gp_" prefix that the frontend adds
      const bridgeModelId = wan2gpModelId.replace(/^wan2gp_/, "");
      getWan2gpProfiles(bridgeModelId)
        .then((resp) => {
          setProfiles(resp.profiles);
          setProfilesLoading(false);
        })
        .catch((e) => {
          console.error("Failed to load profiles:", e);
          setProfiles([]);
          setProfilesLoading(false);
        });
    } else {
      setProfiles([]);
    }
  }, [isWan2gp, isOpen, wan2gpModelId]);

  // Reset loaded flag when provider changes
  useEffect(() => {
    setLoaded(false);
    setSelectedProfileName(null);
  }, [selectedProvider]);

  // Measure content height for smooth animation
  useEffect(() => {
    if (contentRef.current) {
      setContentHeight(contentRef.current.scrollHeight);
    }
  }, [isOpen, settings, loaded, profiles, profilesLoading, selectedProfileName, lengthMode]);

  // Debounced save
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const saveSettings = useCallback(
    (patch: UpdateWan2gpSettingsRequest) => {
      setSettings((prev) => {
        if (!prev) return prev;
        const updated = { ...prev, ...patch };
        if (saveTimeoutRef.current) clearTimeout(saveTimeoutRef.current);
        saveTimeoutRef.current = setTimeout(() => {
          updateWan2gpSettings(patch).catch((e) =>
            console.error("Failed to save Wan2GP settings:", e),
          );
        }, 300);
        return updated;
      });
    },
    [],
  );

  const handleProfileSelect = (profileName: string) => {
    if (profileName === "") {
      setSelectedProfileName(null);
      setSettings((prev) =>
        prev ? { ...prev, profile_params: null } : prev,
      );
      updateWan2gpSettings({}).catch((e) =>
        console.error("Failed to clear profile:", e),
      );
      return;
    }

    const profile = profiles.find((p) => p.name === profileName);
    if (!profile) return;

    setSelectedProfileName(profileName);

    const params = profile.params as Record<string, unknown>;
    updateWan2gpSettings({
      profile_params: params as any,
      ...(typeof params.num_inference_steps === "number"
        ? { num_inference_steps: params.num_inference_steps }
        : {}),
      ...(typeof params.guidance_scale === "number"
        ? { guidance_scale: params.guidance_scale }
        : {}),
    }).catch((e) =>
      console.error("Failed to apply profile:", e),
    );

    setSettings((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        profile_params: params as any,
        ...(typeof params.num_inference_steps === "number"
          ? { num_inference_steps: params.num_inference_steps }
          : {}),
        ...(typeof params.guidance_scale === "number"
          ? { guidance_scale: params.guidance_scale }
          : {}),
      };
    });
  };

  // Video length helpers
  const currentFrames = settings?.video_length ?? DEFAULT_FRAMES;
  const currentSeconds = currentFrames / fps;

  const handleSecondsChange = (seconds: number) => {
    const frames = Math.round(seconds * fps);
    saveSettings({ video_length: frames });
  };

  const handleFramesChange = (frames: number) => {
    saveSettings({ video_length: frames });
  };

  if (!isWan2gp) return null;

  return (
    <div className="mt-2 overflow-hidden rounded-lg border border-white/[0.06] bg-white/[0.03]">
      {/* Toggle header */}
      <button
        onClick={() => setIsOpen((prev) => !prev)}
        className="flex w-full items-center gap-2 px-3 py-2 text-xs font-medium text-base-fg/70 transition-colors hover:text-base-fg cursor-pointer"
      >
        <FontAwesomeIcon icon={faGear} className="h-3 w-3 opacity-60" />
        <span>Advanced Settings</span>
        <FontAwesomeIcon
          icon={faChevronDown}
          className={twMerge(
            "ml-auto h-2.5 w-2.5 opacity-50 transition-transform duration-200",
            isOpen && "rotate-180",
          )}
        />
      </button>

      {/* Collapsible content */}
      <div
        className="transition-all duration-200 ease-in-out"
        style={{
          maxHeight: isOpen ? contentHeight + 16 : 0,
          opacity: isOpen ? 1 : 0,
        }}
      >
        <div ref={contentRef} className="space-y-3 px-3 pb-3">
          {!loaded ? (
            <div className="flex items-center gap-2 py-2 text-xs text-base-fg/40">
              <div className="h-3 w-3 animate-spin rounded-full border-2 border-white/20 border-t-white/60" />
              Loading settings…
            </div>
          ) : settings ? (
            <>
              {/* Accelerator Profile */}
              {profiles.length > 0 && (
                <SettingRow
                  label="Accelerator"
                  tooltip="Speed profile — applies LoRA and step optimizations"
                >
                  <div className="flex items-center gap-1.5">
                    <FontAwesomeIcon
                      icon={faBolt}
                      className="h-3 w-3 text-yellow-400/70"
                    />
                    <select
                      value={selectedProfileName ?? ""}
                      onChange={(e) => handleProfileSelect(e.target.value)}
                      className="max-w-[180px] truncate rounded bg-white/[0.06] px-2 py-1 text-xs text-base-fg outline-none ring-1 ring-white/10 transition-colors focus:ring-primary/50 cursor-pointer"
                    >
                      <option value="">None (default)</option>
                      {profiles.map((p) => (
                        <option key={p.name} value={p.name}>
                          {p.name}
                        </option>
                      ))}
                    </select>
                  </div>
                </SettingRow>
              )}
              {profilesLoading && (
                <div className="flex items-center gap-2 text-xs text-base-fg/40">
                  <div className="h-2.5 w-2.5 animate-spin rounded-full border-2 border-white/20 border-t-white/60" />
                  Loading profiles…
                </div>
              )}

              {/* Seed */}
              <SettingRow label="Seed" tooltip="Random seed (-1 = random)">
                <div className="flex items-center gap-1.5">
                  <input
                    type="number"
                    value={settings.seed ?? -1}
                    onChange={(e) =>
                      saveSettings({ seed: parseInt(e.target.value) || -1 })
                    }
                    className="w-24 rounded bg-white/[0.06] px-2 py-1 text-xs text-base-fg outline-none ring-1 ring-white/10 transition-colors focus:ring-primary/50"
                  />
                  <button
                    onClick={() => saveSettings({ seed: -1 })}
                    className="rounded p-1 text-base-fg/40 transition-colors hover:bg-white/10 hover:text-base-fg cursor-pointer"
                    title="Randomize"
                  >
                    <FontAwesomeIcon icon={faDice} className="h-3 w-3" />
                  </button>
                </div>
              </SettingRow>

              {/* Inference Steps */}
              <SettingRow
                label="Inference Steps"
                tooltip="More steps = higher quality, slower"
              >
                <SliderInput
                  value={settings.num_inference_steps ?? 25}
                  min={1}
                  max={100}
                  step={1}
                  onChange={(v) => saveSettings({ num_inference_steps: v })}
                />
              </SettingRow>

              {/* Guidance Scale */}
              <SettingRow
                label="Guidance Scale"
                tooltip="How closely to follow the prompt"
              >
                <SliderInput
                  value={settings.guidance_scale ?? 7.0}
                  min={0}
                  max={20}
                  step={0.5}
                  onChange={(v) => saveSettings({ guidance_scale: v })}
                />
              </SettingRow>

              {/* Video Length — only for video */}
              {!hideVideoFields && (
                <div className="space-y-1.5">
                  <div className="flex items-center justify-between gap-4">
                    <div className="flex items-center gap-2">
                      <span
                        className="text-xs text-base-fg/60 select-none"
                        title="Duration of generated video"
                      >
                        Video Length
                      </span>
                      <span className="text-[10px] text-base-fg/30">
                        {fps} fps
                      </span>
                    </div>
                    {/* Seconds / Frames toggle */}
                    <div className="flex items-center rounded-md bg-white/[0.04] p-0.5">
                      <button
                        onClick={() => setLengthMode("seconds")}
                        className={twMerge(
                          "rounded px-2 py-0.5 text-[10px] font-medium transition-all cursor-pointer",
                          lengthMode === "seconds"
                            ? "bg-primary/20 text-primary"
                            : "text-base-fg/40 hover:text-base-fg/60",
                        )}
                      >
                        Seconds
                      </button>
                      <button
                        onClick={() => setLengthMode("frames")}
                        className={twMerge(
                          "rounded px-2 py-0.5 text-[10px] font-medium transition-all cursor-pointer",
                          lengthMode === "frames"
                            ? "bg-primary/20 text-primary"
                            : "text-base-fg/40 hover:text-base-fg/60",
                        )}
                      >
                        Frames
                      </button>
                    </div>
                  </div>

                  {lengthMode === "seconds" ? (
                    <div className="flex items-center justify-between">
                      <span className="text-[10px] text-base-fg/30">
                        {currentFrames} frames
                      </span>
                      <SliderInput
                        value={Math.round(currentSeconds * 10) / 10}
                        min={0.5}
                        max={12}
                        step={0.5}
                        onChange={handleSecondsChange}
                        suffix="s"
                      />
                    </div>
                  ) : (
                    <div className="flex items-center justify-between">
                      <span className="text-[10px] text-base-fg/30">
                        ≈ {(currentFrames / fps).toFixed(1)}s
                      </span>
                      <SliderInput
                        value={currentFrames}
                        min={1}
                        max={200}
                        step={1}
                        onChange={handleFramesChange}
                      />
                    </div>
                  )}
                </div>
              )}
            </>
          ) : null}
        </div>
      </div>
    </div>
  );
};

/* ── Subcomponents ──────────────────────────────────────────────────── */

const SettingRow = ({
  label,
  tooltip,
  children,
}: {
  label: string;
  tooltip?: string;
  children: React.ReactNode;
}) => (
  <div className="flex items-center justify-between gap-4">
    <span
      className="text-xs text-base-fg/60 select-none"
      title={tooltip}
    >
      {label}
    </span>
    {children}
  </div>
);

const SliderInput = ({
  value,
  min,
  max,
  step,
  onChange,
  suffix,
}: {
  value: number;
  min: number;
  max: number;
  step: number;
  onChange: (value: number) => void;
  suffix?: string;
}) => {
  return (
    <div className="flex items-center gap-2">
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        className="h-1 w-24 cursor-pointer appearance-none rounded-full bg-white/10 accent-primary [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-primary"
      />
      <div className="flex items-center">
        <input
          type="number"
          value={value}
          min={min}
          max={max}
          step={step}
          onChange={(e) => {
            const v = parseFloat(e.target.value);
            if (!isNaN(v)) onChange(v);
          }}
          className="w-16 rounded bg-white/[0.06] px-2 py-1 text-right text-xs text-base-fg outline-none ring-1 ring-white/10 transition-colors focus:ring-primary/50"
        />
        {suffix && (
          <span className="ml-1 text-[10px] text-base-fg/40">{suffix}</span>
        )}
      </div>
    </div>
  );
};
