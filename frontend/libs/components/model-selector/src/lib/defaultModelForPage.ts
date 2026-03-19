import {
  Model,
  SPLAT_MODELS_BY_ID,
  VIDEO_MODELS_BY_ID,
} from "@storyteller/model-list";
import { ModelPage } from "./model-pages";

export const defaultModelForPage = (
  models: Model[],
  page: ModelPage,
): Model => {
  let imageModel: Model | undefined;

  switch (page) {
    // NB: Cloud models disabled — fall through to first available local model.
    // Old defaults kept as comments for reference:
    // case ModelPage.TextToImage:  imageModel = IMAGE_MODELS_BY_ID.get("nano_banana_pro");
    // case ModelPage.Canvas2D:     imageModel = IMAGE_MODELS_BY_ID.get("gpt_image_1p5");
    // case ModelPage.Stage3D:      imageModel = IMAGE_MODELS_BY_ID.get("gpt_image_1p5");
    // case ModelPage.ImageEditor:  imageModel = IMAGE_MODELS_BY_ID.get("nano_banana_pro");
    // case ModelPage.Angles:       imageModel = IMAGE_MODELS_BY_ID.get("flux_2_lora_angles");
    case ModelPage.TextToImage:
    case ModelPage.Canvas2D:
    case ModelPage.Stage3D:
    case ModelPage.ImageEditor:
    case ModelPage.Angles:
      break; // Falls through to models[0]
    case ModelPage.ImageToVideo:
      imageModel = VIDEO_MODELS_BY_ID.get("wan2gp_local");
      break;
    case ModelPage.ImageTo3DWorld:
      imageModel = SPLAT_MODELS_BY_ID.get("marble_0p1_mini");
      break;
  }

  return imageModel || models[0];
};
