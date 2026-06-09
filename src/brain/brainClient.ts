import { invoke } from "@tauri-apps/api/core";
import type { AppleNotesFolder, AppleNotesSource, BrainStore } from "./types";

export function loadBrainStore(): Promise<BrainStore> {
  return invoke<BrainStore>("load_brain_store");
}

export function listAppleNotesFolders(): Promise<AppleNotesFolder[]> {
  return invoke<AppleNotesFolder[]>("list_apple_notes_folders");
}

export function ensureZoidBrainFolder(): Promise<AppleNotesSource> {
  return invoke<AppleNotesSource>("ensure_zoid_brain_folder");
}

export function linkAppleNotesFolder(accountName: string, folderName: string, syncMode: string): Promise<AppleNotesSource> {
  return invoke<AppleNotesSource>("link_apple_notes_folder", { accountName, folderName, syncMode });
}

export function syncAppleNotesSources(): Promise<BrainStore> {
  return invoke<BrainStore>("sync_apple_notes_sources");
}

export function extractBrainNote(noteId: string): Promise<BrainStore> {
  return invoke<BrainStore>("extract_brain_note", { noteId });
}

export function createBrainClarifyingSession(noteId: string, taskCandidateIds: string[]): Promise<BrainStore> {
  return invoke<BrainStore>("create_brain_clarifying_session", { noteId, taskCandidateIds });
}

export function answerBrainClarifyingSession(sessionId: string, answer: string): Promise<BrainStore> {
  return invoke<BrainStore>("answer_brain_clarifying_session", { sessionId, answer });
}
