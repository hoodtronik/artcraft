import { useState, useEffect, useRef, useCallback } from "react";
import { faChevronDown, faGear, faDice } from "@fortawesome/pro-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { GenerationProvider } from "@storyteller/api-enums";
import {
  getWan2gpSettings,
  updateWan2gpSettings,
  type Wan2gpSettings,
  type UpdateWan2gpSettingsRequest,
} from "@storyteller/tauri-api";
import { twMerge } from "tailwind-merge";

interface Wan2gpAdvancedSettingsProps {
  selectedProvider?: GenerationProvider;
  /** Hide video-only fields (e.g. video length) */
  hideVideoFields?: boolean;
}

/**
 * Collapsible "Advanced Settings" panel for Wan2GP local generation.
 * Only renders when selectedProvider === GenerationProvider.Wan2gp.
 */
export const Wan2gpAdvancedSettings = ({
  selectedProvider,
  hideVideoFields,
}: Wan2gpAdvancedSettingsProps) => {
  const [isOpen, setIsOpen] = useState(false);
  const [settings, setSettings] = useState<Wan2gpSettings | null>(null);
  const [loaded, setLoaded] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);
  const [contentHeight, setContentHeight] = useState(0);

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

  // Reset loaded flag when provider changes
  useEffect(() => {
    setLoaded(false);
  }, [selectedProvider]);

  // Measure content height for smooth animation
  useEffect(() => {
    if (contentRef.current) {
      setContentHeight(contentRef.current.scrollHeight);
    }
  }, [isOpen, settings, loaded]);

  // Debounced save
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const saveSettings = useCallback(
    (patch: UpdateWan2gpSettingsRequest) => {
      setSettings((prev) => {
        if (!prev) return prev;
        const updated = { ...prev, ...patch };
        // Debounce the Tauri call
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
                  placeholder="auto"
                  isDefault={settings.num_inference_steps === null}
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
                  placeholder="auto"
                  isDefault={settings.guidance_scale === null}
                />
              </SettingRow>

              {/* Video Length — only for video */}
              {!hideVideoFields && (
                <SettingRow
                  label="Video Length"
                  tooltip="Number of frames to generate"
                >
                  <SliderInput
                    value={settings.video_length ?? 49}
                    min={1}
                    max={200}
                    step={1}
                    onChange={(v) => saveSettings({ video_length: v })}
                    placeholder="auto"
                    isDefault={settings.video_length === null}
                  />
                </SettingRow>
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
  placeholder,
  isDefault,
}: {
  value: number;
  min: number;
  max: number;
  step: number;
  onChange: (value: number) => void;
  placeholder?: string;
  isDefault?: boolean;
}) => {
  const displayValue = isDefault ? "" : value;

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
      <input
        type="number"
        value={displayValue}
        placeholder={placeholder}
        min={min}
        max={max}
        step={step}
        onChange={(e) => {
          const v = parseFloat(e.target.value);
          if (!isNaN(v)) onChange(v);
        }}
        className="w-16 rounded bg-white/[0.06] px-2 py-1 text-right text-xs text-base-fg outline-none ring-1 ring-white/10 transition-colors focus:ring-primary/50"
      />
    </div>
  );
};
