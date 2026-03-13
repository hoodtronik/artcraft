import { Button } from "@storyteller/ui-button";
import { useEffect, useState } from "react";
import { faSpinnerThird, faCheck, faXmark, faRefresh } from "@fortawesome/pro-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  getWan2gpStatus,
  getWan2gpSettings,
  getWan2gpModels,
  updateWan2gpSettings,
  Wan2gpStatus,
  Wan2gpSettings,
  Wan2gpModelInfo,
} from "@storyteller/tauri-api";

export const Wan2gpAccountBlock = () => {
  const [status, setStatus] = useState<Wan2gpStatus | undefined>(undefined);
  const [settings, setSettings] = useState<Wan2gpSettings | undefined>(undefined);
  const [models, setModels] = useState<Wan2gpModelInfo[]>([]);
  const [isChecking, setIsChecking] = useState(false);
  const [bridgeUrlInput, setBridgeUrlInput] = useState("http://localhost:7861");
  const [showModels, setShowModels] = useState(false);

  const fetchAll = async () => {
    setIsChecking(true);
    try {
      const [statusResult, settingsResult] = await Promise.all([
        getWan2gpStatus(),
        getWan2gpSettings(),
      ]);
      setStatus(statusResult);
      setSettings(settingsResult);
      setBridgeUrlInput(settingsResult.bridge_url);

      if (statusResult.online) {
        const modelsResult = await getWan2gpModels();
        setModels(modelsResult.video);
      }
    } catch (e) {
      console.error("Error fetching Wan2GP info", e);
    } finally {
      setIsChecking(false);
    }
  };

  useEffect(() => {
    fetchAll();
  }, []);

  const handleSaveBridgeUrl = async () => {
    try {
      await updateWan2gpSettings({ bridge_url: bridgeUrlInput });
      await fetchAll();
    } catch (e) {
      console.error("Error saving bridge URL", e);
    }
  };

  const handleSelectModel = async (modelId: string) => {
    try {
      await updateWan2gpSettings({ selected_model: modelId });
      const updated = await getWan2gpSettings();
      setSettings(updated);
    } catch (e) {
      console.error("Error selecting model", e);
    }
  };

  return (
    <div className="rounded-lg border border-white/10 p-4 space-y-3">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div className="flex items-center gap-2">
          <span className="text-lg font-semibold">🖥️ Local GPU (Wan2GP)</span>
          {status?.online ? (
            <span className="flex items-center gap-1 rounded-full bg-green-500/20 px-2 py-0.5 text-xs text-green-400">
              <FontAwesomeIcon icon={faCheck} className="text-[10px]" />
              Online
            </span>
          ) : (
            <span className="flex items-center gap-1 rounded-full bg-red-500/20 px-2 py-0.5 text-xs text-red-400">
              <FontAwesomeIcon icon={faXmark} className="text-[10px]" />
              Offline
            </span>
          )}
        </div>
        <Button
          variant="secondary"
          className="h-[30px]"
          onClick={fetchAll}
          disabled={isChecking}
        >
          {isChecking ? (
            <FontAwesomeIcon
              icon={faSpinnerThird}
              className="animate-spin text-sm"
            />
          ) : (
            <FontAwesomeIcon icon={faRefresh} className="text-sm" />
          )}
        </Button>
      </div>

      {/* GPU Info */}
      {status?.online && status?.gpu && (
        <div className="text-sm text-white/60 space-y-1">
          <div>GPU: <span className="text-white/80">{status.gpu}</span></div>
          {status.gpu_vram_gb && (
            <div>VRAM: <span className="text-white/80">{status.gpu_vram_gb.toFixed(1)} GB</span></div>
          )}
          {status.engine && (
            <div>Engine: <span className="text-white/80">{status.engine}</span></div>
          )}
        </div>
      )}

      {/* Bridge URL */}
      <div className="space-y-1">
        <label className="text-sm text-white/60">Bridge URL</label>
        <div className="flex gap-2">
          <input
            type="text"
            value={bridgeUrlInput}
            onChange={(e) => setBridgeUrlInput(e.target.value)}
            className="flex-1 rounded-md bg-white/5 border border-white/10 px-3 py-1.5 text-sm text-white/90 focus:border-purple-500/50 focus:outline-none"
            placeholder="http://localhost:7861"
          />
          <Button
            variant="secondary"
            className="h-[34px] px-3 text-sm"
            onClick={handleSaveBridgeUrl}
          >
            Save
          </Button>
        </div>
      </div>

      {/* Selected Model */}
      {status?.online && (
        <div className="space-y-1">
          <div className="flex items-center justify-between">
            <label className="text-sm text-white/60">
              Selected Model {models.length > 0 && `(${models.length} available)`}
            </label>
            <button
              onClick={() => setShowModels(!showModels)}
              className="text-xs text-purple-400 hover:text-purple-300"
            >
              {showModels ? "Hide models" : "Browse models"}
            </button>
          </div>
          <div className="text-sm text-white/80">
            {settings?.selected_model || "None selected — click 'Browse models'"}
          </div>
        </div>
      )}

      {/* Model List */}
      {showModels && models.length > 0 && (
        <div className="max-h-48 overflow-y-auto rounded-md border border-white/10 bg-black/20">
          {models.map((model) => (
            <button
              key={model.id}
              onClick={() => handleSelectModel(model.id)}
              className={`w-full text-left px-3 py-2 text-sm hover:bg-white/5 border-b border-white/5 last:border-b-0 transition-colors ${
                settings?.selected_model === model.id
                  ? "bg-purple-500/10 text-purple-300"
                  : "text-white/80"
              }`}
            >
              <div className="font-medium">{model.name}</div>
              <div className="text-xs text-white/40">
                {model.architecture}
                {model.is_i2v && " • Image-to-Video"}
              </div>
            </button>
          ))}
        </div>
      )}

      {/* Offline help */}
      {!status?.online && (
        <div className="rounded-md bg-white/5 p-3 text-xs text-white/50">
          <p>
            Start Wan2GP with the ArtCraft Bridge plugin to enable local GPU generation.
            No account or API key needed — it's free and runs on your hardware.
          </p>
          {status?.error && (
            <p className="mt-1 text-red-400/70">Error: {status.error}</p>
          )}
        </div>
      )}
    </div>
  );
};
