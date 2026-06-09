export type RepositorySource = "scanned" | "cloned";

export type CodeRepository = {
  id: string;
  name: string;
  path: string;
  remoteUrl?: string;
  branch?: string;
  defaultBranch?: string;
  dirty: boolean;
  latestCommit?: {
    hash: string;
    message: string;
    date: string;
  };
  addedAt: string;
  source: RepositorySource;
};

export type GithubBranch = {
  name: string;
  isDefault: boolean;
};
