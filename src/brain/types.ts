export type BrainSyncMode = "twoWay" | "readOnly" | "ignored";
export type BrainSyncStatus = "synced" | "changedInApple" | "changedInZoid" | "conflict" | "missingInApple" | "writeFailed" | "unlinked";

export type BrainStore = {
  version: 1;
  sources: AppleNotesSource[];
  notes: BrainNote[];
  extractions: BrainExtraction[];
  taskCandidates: TaskCandidate[];
  clarificationSessions: BrainClarificationSession[];
  conflicts: BrainSyncConflict[];
  updatedAt: string;
};

export type AppleNotesSource = {
  id: string;
  sourceType: "appleNotes";
  accountName: string;
  folderName: string;
  syncMode: BrainSyncMode;
  enabled: boolean;
  createdByZoid: boolean;
  lastSyncedAt: string | null;
  lastError: string | null;
};

export type BrainNote = {
  id: string;
  sourceType: "appleNotes";
  sourceId: string;
  appleNoteId: string;
  title: string;
  body: string;
  sourceFolder: string;
  accountName: string;
  appleCreatedAt: string | null;
  appleModifiedAt: string | null;
  zoidModifiedAt: string | null;
  importedAt: string;
  lastSyncedAt: string | null;
  lastSyncedTitle: string;
  lastSyncedBody: string;
  lastSyncedHash: string;
  currentHash: string;
  syncStatus: BrainSyncStatus;
  archived: boolean;
};

export type BrainExtraction = {
  id: string;
  noteId: string;
  summary: string;
  topics: string[];
  entities: string[];
  references: string[];
  decisions: string[];
  openQuestions: string[];
  ambiguityScore: number;
  extractedAt: string;
  extractor: "localHeuristic" | "hermes";
};

export type TaskCandidate = {
  id: string;
  noteId: string;
  title: string;
  extractedDescription: string;
  status: "needsReview" | "needsClarification" | "readyForAgent" | "sentToAgent" | "done" | "rejected" | "merged";
  priorityGuess: "low" | "normal" | "high";
  readinessScore: number;
  clarificationSessionId: string | null;
  createdAt: string;
  updatedAt: string;
};

export type BrainClarificationSession = {
  id: string;
  noteId: string;
  taskCandidateIds: string[];
  status: "draft" | "questioning" | "briefReady" | "sentToAgent" | "archived";
  transcript: Array<{ role: "user" | "assistant"; content: string; createdAt: string }>;
  resolvedBrief: string;
  openQuestions: string[];
  hermesSessionId: string | null;
  createdAt: string;
  updatedAt: string;
};

export type BrainSyncConflict = {
  id: string;
  noteId: string;
  appleTitle: string;
  appleBody: string;
  zoidTitle: string;
  zoidBody: string;
  detectedAt: string;
  resolvedAt: string | null;
  resolution: "keepApple" | "keepZoid" | "manualMerge" | "saveBoth" | null;
};

export type AppleNotesFolder = {
  id: string | null;
  accountName: string;
  folderName: string;
};
