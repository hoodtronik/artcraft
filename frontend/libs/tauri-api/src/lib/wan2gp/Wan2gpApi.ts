import { invoke } from "@tauri-apps/api/core";

// ── Types ──────────────────────────────────────────────────────────────────

export interface Wan2gpSettings {
  bridge_url: string;
  selected_model: string | null;
  resolution: string | null;
  seed: number | null;
  num_inference_steps: number | null;
  guidance_scale: number | null;
  video_length: number | null;
}

export interface Wan2gpStatus {
  online: boolean;
  engine: string | null;
  gpu: string | null;
  gpu_vram_gb: number | null;
  active_tasks: number | null;
  error: string | null;
}

export interface Wan2gpModelInfo {
  id: string;
  name: string;
  architecture: string;
  model_type: string; // "video" or "image"
  is_i2v: boolean;
  description: string;
}

export interface Wan2gpModelsResponse {
  video: Wan2gpModelInfo[];
  image: Wan2gpModelInfo[];
  error: string | null;
}

export interface UpdateWan2gpSettingsRequest {
  bridge_url?: string;
  selected_model?: string;
  resolution?: string;
  seed?: number;
  num_inference_steps?: number;
  guidance_scale?: number;
  video_length?: number;
  profile_params?: Record<string, unknown>;
}

// ── API calls ──────────────────────────────────────────────────────────────

export const getWan2gpSettings = async (): Promise<Wan2gpSettings> => {
  return (await invoke("wan2gp_get_settings_command")) as Wan2gpSettings;
};

export const updateWan2gpSettings = async (
  request: UpdateWan2gpSettingsRequest,
): Promise<void> => {
  await invoke("wan2gp_update_settings_command", { request });
};

export const getWan2gpStatus = async (): Promise<Wan2gpStatus> => {
  return (await invoke("wan2gp_get_status_command")) as Wan2gpStatus;
};

export const getWan2gpModels = async (): Promise<Wan2gpModelsResponse> => {
  return (await invoke("wan2gp_get_models_command")) as Wan2gpModelsResponse;
};

export interface Wan2gpCancelResponse {
  success: boolean;
  message: string | null;
}

export const cancelWan2gpTask = async (
  taskId: string,
): Promise<Wan2gpCancelResponse> => {
  return (await invoke("wan2gp_cancel_task_command", {
    taskId,
  })) as Wan2gpCancelResponse;
};

// ── Profiles ───────────────────────────────────────────────────────────────

export interface Wan2gpProfileInfo {
  name: string;
  params: Record<string, unknown>;
}

export interface Wan2gpProfilesResponse {
  profiles: Wan2gpProfileInfo[];
  error: string | null;
}

export const getWan2gpProfiles = async (
  modelId: string,
): Promise<Wan2gpProfilesResponse> => {
  return (await invoke("wan2gp_get_profiles_command", {
    modelId,
  })) as Wan2gpProfilesResponse;
};
