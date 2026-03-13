
// NOTE: These are defined in Rust (as the source of truth) and duplicated in the frontend.
// In the future, we should use code gen (protobufs or similar) to keep the two sides in sync.

export enum GenerationProvider {
  Artcraft = "artcraft",
  Grok = "grok",
  Midjourney = "midjourney",
  Sora = "sora",
  Wan2gp = "wan2gp",
  WorldLabs = "world_labs",
  // NB: We should build our own open source Fal
  //Fal = "fal",
}
