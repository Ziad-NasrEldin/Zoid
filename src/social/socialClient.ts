import { invoke } from "@tauri-apps/api/core";
import type { MavoidMediaValidation, MavoidSocialJobResult, MavoidSocialOverview, MavoidSocialPost } from "./types";

export async function getMavoidSocialOverview(): Promise<MavoidSocialOverview> {
  return invoke<MavoidSocialOverview>("mavoid_social_get_overview");
}

export async function listMavoidSocialPosts(): Promise<MavoidSocialPost[]> {
  return invoke<MavoidSocialPost[]>("mavoid_social_list_posts");
}

export async function getMavoidSocialPost(postId: string): Promise<MavoidSocialPost> {
  return invoke<MavoidSocialPost>("mavoid_social_get_post", { postId });
}

export async function runMavoidBufferHealthCheck(): Promise<MavoidSocialOverview> {
  return invoke<MavoidSocialOverview>("mavoid_social_run_buffer_health_check");
}

export async function manageMavoidSocialAutomation(action: string): Promise<MavoidSocialOverview> {
  return invoke<MavoidSocialOverview>("mavoid_social_manage_automation", { action });
}

export async function validateMavoidMediaUrl(url: string): Promise<MavoidMediaValidation> {
  return invoke<MavoidMediaValidation>("mavoid_social_validate_media_url", { url });
}

export async function openMavoidSocialResource(resource: string): Promise<void> {
  await invoke("mavoid_social_open_resource", { resource });
}

export async function startMavoidSocialPostGeneration(date: string, contentType: string): Promise<MavoidSocialJobResult> {
  return invoke<MavoidSocialJobResult>("mavoid_social_start_generation", { date, contentType });
}

export async function retryMavoidSocialDesign(postId: string, mediaPath: string, notes: string): Promise<MavoidSocialJobResult> {
  return invoke<MavoidSocialJobResult>("mavoid_social_retry_design", { postId, mediaPath, notes });
}
