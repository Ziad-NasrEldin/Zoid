import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { CodeRepository, GithubBranch } from "./types";

export async function selectFolderFromFinder(title: string): Promise<string | null> {
  const selected = await open({ directory: true, multiple: false, recursive: true, title });
  return typeof selected === "string" ? selected : null;
}

export function scanGithubRepositories(folder: string): Promise<CodeRepository[]> {
  return invoke<CodeRepository[]>("scan_github_repositories", { folder });
}

export function cloneGithubRepository(repoUrl: string, destinationRoot: string): Promise<CodeRepository> {
  return invoke<CodeRepository>("clone_github_repository", { repoUrl, destinationRoot });
}

export function listGithubBranches(repositoryPath: string, remoteUrl?: string | null, currentDefaultBranch?: string | null): Promise<GithubBranch[]> {
  return invoke<GithubBranch[]>("list_github_branches", { repositoryPath, remoteUrl, currentDefaultBranch });
}

export function updateGithubDefaultBranch(repositoryPath: string, remoteUrl: string | null | undefined, defaultBranch: string): Promise<CodeRepository> {
  return invoke<CodeRepository>("update_github_default_branch", { repositoryPath, remoteUrl, defaultBranch });
}
