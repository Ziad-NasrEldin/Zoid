import { Search, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { GlobalDropdown } from "../ui/GlobalDropdown";
import { cloneGithubRepository, listGithubBranches, scanGithubRepositories, selectFolderFromFinder, updateGithubDefaultBranch } from "./repositoryClient";
import type { CodeRepository, GithubBranch } from "./types";

type CodeWorkspaceProps = {
  repositories: CodeRepository[];
  onRepositoriesChange: (repositories: CodeRepository[]) => void;
  linkedRepositoryId: string;
  onLinkedRepositoryIdChange: (repositoryId: string) => void;
};

const DEFAULT_SCAN_FOLDER = "";
const DEFAULT_CLONE_ROOT = "";

function errorToMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error || "Unknown error");
}

function mergeRepositories(current: CodeRepository[], incoming: CodeRepository[]) {
  const merged = new Map<string, CodeRepository>();
  for (const repository of current) merged.set(repository.id, repository);
  for (const repository of incoming) {
    const existing = merged.get(repository.id);
    merged.set(repository.id, existing ? { ...existing, ...repository, addedAt: existing.addedAt } : repository);
  }
  return Array.from(merged.values()).sort((left, right) => left.name.localeCompare(right.name) || left.path.localeCompare(right.path));
}

function RepositoryMeta({
  repository,
  isUpdatingDefaultBranch,
  isEditingDefaultBranch,
  branchOptions,
  selectedDefaultBranch,
  onSelectedDefaultBranchChange,
  onEditDefaultBranch,
  onCancelDefaultBranchEdit,
  onSaveDefaultBranch,
}: {
  repository: CodeRepository;
  isUpdatingDefaultBranch: boolean;
  isEditingDefaultBranch: boolean;
  branchOptions: GithubBranch[];
  selectedDefaultBranch: string;
  onSelectedDefaultBranchChange: (branch: string) => void;
  onEditDefaultBranch: (repository: CodeRepository) => void;
  onCancelDefaultBranchEdit: () => void;
  onSaveDefaultBranch: (repository: CodeRepository) => void;
}) {
  const defaultBranch = repository.defaultBranch || repository.branch || "Detached / unknown";
  const latestCommitDate = repository.latestCommit?.date ?? "No commits detected";
  return (
    <dl className="repo-meta-grid">
      <div>
        <dt>Path</dt>
        <dd title={repository.path}>{repository.path}</dd>
      </div>
      <div>
        <dt>Remote</dt>
        <dd title={repository.remoteUrl ?? "No origin remote"}>{repository.remoteUrl ?? "No origin remote"}</dd>
      </div>
      <div className={`repo-meta-grid-item repo-meta-grid-item--default-branch${isEditingDefaultBranch ? " repo-meta-grid-item--editing" : ""}`}>
        <dt>Default branch</dt>
        <dd className={`repo-meta-action-row${isEditingDefaultBranch ? " repo-meta-action-row--editing" : ""}`}>
          {isEditingDefaultBranch ? (
            <div className="default-branch-editor" aria-label="Edit default branch">
              <label className="sr-only" htmlFor={`default-branch-${repository.id}`}>Select a default branch</label>
              <GlobalDropdown
                className="default-branch-dropdown"
                disabled={isUpdatingDefaultBranch}
                id={`default-branch-${repository.id}`}
                label="Select a default branch"
                onChange={onSelectedDefaultBranchChange}
                options={branchOptions.map((branch) => ({
                  value: branch.name,
                  label: branch.name,
                  meta: branch.isDefault ? "current default" : undefined,
                }))}
                size="compact"
                value={selectedDefaultBranch}
              />
              <button className="default-branch-save-button" disabled={isUpdatingDefaultBranch || !selectedDefaultBranch || selectedDefaultBranch === defaultBranch} onClick={() => onSaveDefaultBranch(repository)} type="button">
                {isUpdatingDefaultBranch ? "Saving…" : "Save"}
              </button>
              <button className="default-branch-cancel-button" disabled={isUpdatingDefaultBranch} onClick={onCancelDefaultBranchEdit} type="button">Cancel</button>
            </div>
          ) : (
            <>
              <span>{defaultBranch}</span>
              <button disabled={isUpdatingDefaultBranch} onClick={() => onEditDefaultBranch(repository)} type="button">
                {isUpdatingDefaultBranch ? "Loading…" : "Edit"}
              </button>
            </>
          )}
        </dd>
      </div>
      <div>
        <dt>Status</dt>
        <dd>{repository.dirty ? "Dirty" : "Clean"}</dd>
      </div>
      <div>
        <dt>Latest commit</dt>
        <dd title={repository.latestCommit ? `${repository.latestCommit.hash} ${repository.latestCommit.message}` : latestCommitDate}>{latestCommitDate}</dd>
      </div>
      <div>
        <dt>Source</dt>
        <dd>{repository.source}</dd>
      </div>
    </dl>
  );
}

export function CodeWorkspace({
  repositories,
  onRepositoriesChange,
  linkedRepositoryId,
  onLinkedRepositoryIdChange,
}: CodeWorkspaceProps) {
  const [scanFolder, setScanFolder] = useState(DEFAULT_SCAN_FOLDER);
  const [repoUrl, setRepoUrl] = useState("");
  const [destinationRoot, setDestinationRoot] = useState(DEFAULT_CLONE_ROOT);
  const [busyAction, setBusyAction] = useState<"scan" | "clone" | null>(null);
  const [isRepositorySearchOpen, setIsRepositorySearchOpen] = useState(false);
  const [repositorySearchQuery, setRepositorySearchQuery] = useState("");
  const [updatingDefaultBranchRepositoryId, setUpdatingDefaultBranchRepositoryId] = useState<string | null>(null);
  const [editingDefaultBranchRepositoryId, setEditingDefaultBranchRepositoryId] = useState<string | null>(null);
  const [defaultBranchOptions, setDefaultBranchOptions] = useState<GithubBranch[]>([]);
  const [selectedDefaultBranch, setSelectedDefaultBranch] = useState("");
  const [defaultBranchStatus, setDefaultBranchStatus] = useState("");
  const [defaultBranchError, setDefaultBranchError] = useState("");
  const [repositoryScanFeedback, setRepositoryScanFeedback] = useState<{ tone: "info" | "success" | "error"; label: string; message: string } | null>(null);
  const [recentlyAddedRepositoryIds, setRecentlyAddedRepositoryIds] = useState<string[]>([]);

  const filteredRepositories = useMemo(() => {
    const query = repositorySearchQuery.trim().toLowerCase();
    if (!query) return repositories;

    return repositories.filter((repository) => {
      const searchableValues = [
        repository.name,
        repository.path,
        repository.remoteUrl ?? "",
        repository.branch ?? "",
        repository.defaultBranch ?? "",
        repository.source,
        repository.dirty ? "dirty" : "clean",
        repository.latestCommit?.hash ?? "",
        repository.latestCommit?.message ?? "",
      ];
      return searchableValues.some((value) => value.toLowerCase().includes(query));
    });
  }, [repositories, repositorySearchQuery]);

  const repositoryCountLabel = repositorySearchQuery.trim()
    ? `${filteredRepositories.length} of ${repositories.length} shown`
    : `${repositories.length} added`;

  useEffect(() => {
    if (recentlyAddedRepositoryIds.length === 0) return undefined;
    const timer = window.setTimeout(() => setRecentlyAddedRepositoryIds([]), 4200);
    return () => window.clearTimeout(timer);
  }, [recentlyAddedRepositoryIds]);

  async function handleChooseScanFolder() {
    try {
      const selectedFolder = await selectFolderFromFinder("Choose a folder to scan for Git repositories");
      if (selectedFolder) {
        setScanFolder(selectedFolder);
      }
    } catch (error) {
      console.error("Folder selection failed", error);
    }
  }

  async function handleChooseCloneDestination() {
    try {
      const selectedFolder = await selectFolderFromFinder("Choose where cloned repositories should be saved");
      if (selectedFolder) {
        setDestinationRoot(selectedFolder);
      }
    } catch (error) {
      console.error("Folder selection failed", error);
    }
  }

  async function handleScanFolder() {
    const folder = scanFolder.trim();
    if (!folder) {
      console.error("Choose a folder from Finder before scanning.");
      return;
    }

    setBusyAction("scan");
    setRepositoryScanFeedback({ tone: "info", label: "Scanning", message: `Scanning ${folder} for Git repositories…` });
    setRecentlyAddedRepositoryIds([]);
    try {
      const detectedRepositories = await scanGithubRepositories(folder);
      const existingRepositoryIds = new Set(repositories.map((repository) => repository.id));
      const newlyAddedRepositories = detectedRepositories.filter((repository) => !existingRepositoryIds.has(repository.id));
      onRepositoriesChange(mergeRepositories(repositories, detectedRepositories));
      setRecentlyAddedRepositoryIds(newlyAddedRepositories.map((repository) => repository.id));
      setRepositoryScanFeedback({
        tone: newlyAddedRepositories.length > 0 ? "success" : "info",
        label: newlyAddedRepositories.length > 0 ? "Repos added" : "Scan complete",
        message: newlyAddedRepositories.length > 0
          ? `${newlyAddedRepositories.length} new ${newlyAddedRepositories.length === 1 ? "repository" : "repositories"} added to the list: ${newlyAddedRepositories.slice(0, 3).map((repository) => repository.name).join(", ")}${newlyAddedRepositories.length > 3 ? "…" : ""}.`
          : detectedRepositories.length > 0
            ? `No new repositories added. ${detectedRepositories.length} existing ${detectedRepositories.length === 1 ? "repository was" : "repositories were"} already in the list.`
            : "No Git repositories found in the selected folder.",
      });
    } catch (error) {
      console.error("Scan failed", error);
      setRepositoryScanFeedback({ tone: "error", label: "Scan failed", message: errorToMessage(error) });
    } finally {
      setBusyAction(null);
    }
  }

  async function handleCloneRepository() {
    const nextRepoUrl = repoUrl.trim();
    const nextDestinationRoot = destinationRoot.trim();
    if (!nextRepoUrl || !nextDestinationRoot) {
      console.error("Enter a GitHub repo link and choose a destination folder from Finder before cloning.");
      return;
    }

    setBusyAction("clone");
    try {
      const clonedRepository = await cloneGithubRepository(nextRepoUrl, nextDestinationRoot);
      onRepositoriesChange(mergeRepositories(repositories, [clonedRepository]));
      onLinkedRepositoryIdChange(clonedRepository.id);
      setRepoUrl("");
    } catch (error) {
      console.error("Clone failed", error);
    } finally {
      setBusyAction(null);
    }
  }

  async function handleEditDefaultBranch(repository: CodeRepository) {
    const currentDefaultBranch = repository.defaultBranch || repository.branch || "";
    const fallbackBranchOptions = currentDefaultBranch ? [{ name: currentDefaultBranch, isDefault: true }] : [];
    setDefaultBranchOptions(fallbackBranchOptions);
    setSelectedDefaultBranch(currentDefaultBranch);
    setEditingDefaultBranchRepositoryId(repository.id);
    setUpdatingDefaultBranchRepositoryId(repository.id);
    setDefaultBranchError("");
    setDefaultBranchStatus(`Loading GitHub branches for ${repository.name}…`);
    try {
      const branches = await listGithubBranches(repository.path, repository.remoteUrl, currentDefaultBranch);
      const branchNames = new Set(branches.map((branch) => branch.name));
      const options = currentDefaultBranch && !branchNames.has(currentDefaultBranch)
        ? [{ name: currentDefaultBranch, isDefault: true }, ...branches]
        : branches;
      setDefaultBranchOptions(options);
      setSelectedDefaultBranch(currentDefaultBranch || options[0]?.name || "");
      setEditingDefaultBranchRepositoryId(repository.id);
      setDefaultBranchStatus(`Select a default branch for ${repository.name}.`);
    } catch (error) {
      console.error("Default branch selection failed", error);
      setDefaultBranchError(`Default branch selection failed: ${errorToMessage(error)}`);
      setDefaultBranchStatus("Default branch selection failed.");
    } finally {
      setUpdatingDefaultBranchRepositoryId(null);
    }
  }

  function handleCancelDefaultBranchEdit() {
    setEditingDefaultBranchRepositoryId(null);
    setDefaultBranchOptions([]);
    setSelectedDefaultBranch("");
    setDefaultBranchError("");
    setDefaultBranchStatus("Default branch edit cancelled.");
  }

  async function handleSaveDefaultBranch(repository: CodeRepository) {
    const currentDefaultBranch = repository.defaultBranch || repository.branch || "";
    const trimmedDefaultBranch = selectedDefaultBranch.trim();
    if (!trimmedDefaultBranch || trimmedDefaultBranch === currentDefaultBranch) return;

    setUpdatingDefaultBranchRepositoryId(repository.id);
    setDefaultBranchError("");
    setDefaultBranchStatus(`Updating GitHub default branch for ${repository.name}…`);
    try {
      const updatedRepository = await updateGithubDefaultBranch(repository.path, repository.remoteUrl, trimmedDefaultBranch);
      onRepositoriesChange(repositories.map((item) => (item.id === repository.id ? { ...item, ...updatedRepository, addedAt: item.addedAt } : item)));
      setEditingDefaultBranchRepositoryId(null);
      setDefaultBranchOptions([]);
      setSelectedDefaultBranch("");
      setDefaultBranchStatus(`Default branch updated to ${trimmedDefaultBranch}.`);
    } catch (error) {
      console.error("Default branch update failed", error);
      setDefaultBranchError(`Default branch update failed: ${errorToMessage(error)}`);
      setDefaultBranchStatus("Default branch update failed.");
    } finally {
      setUpdatingDefaultBranchRepositoryId(null);
    }
  }

  return (
    <section aria-label="Code workspace" className="code-workspace-shell code-sumi-e">
      <header className="code-workspace-header">
        <div className="code-hero-copy">
          <p className="kana-line">コード</p>
          <h2>GitHub Repositories integration</h2>
          <p>Scan local folders for Git repositories, clone GitHub links, and attach a managed repository to the Hermes agent session.</p>
          <p className="code-reference-line">Native Finder pickers · GitHub branch control · Hermes waits for your repository link</p>
        </div>
        <div className="code-ink-mark" aria-hidden="true"><span /><span /><span /></div>
      </header>

      <div className="repo-control-grid" aria-label="Repository actions">
        <section className="repo-action-panel" aria-label="Scan folder for repositories">
          <h3>Scan folder</h3>
          <label htmlFor="scan-folder-input">Selected folder</label>
          <div className="folder-picker-row">
            <input
              id="scan-folder-input"
              placeholder="Choose a folder from Finder"
              readOnly
              value={scanFolder}
            />
            <button disabled={busyAction !== null} onClick={handleChooseScanFolder} type="button">
              Choose folder…
            </button>
          </div>
          <button disabled={busyAction !== null || !scanFolder.trim()} onClick={handleScanFolder} type="button">
            {busyAction === "scan" ? "Scanning…" : "Scan selected folder"}
          </button>
          {repositoryScanFeedback ? (
            <div className={`repo-action-feedback repo-action-feedback--${repositoryScanFeedback.tone}`} role="status" aria-live="polite">
              <span>{repositoryScanFeedback.label}</span>
              <p>{repositoryScanFeedback.message}</p>
            </div>
          ) : null}
        </section>

        <section className="repo-action-panel" aria-label="Clone repository from GitHub link">
          <h3>Clone repo</h3>
          <label htmlFor="clone-url-input">GitHub repo link</label>
          <input
            id="clone-url-input"
            onChange={(event) => setRepoUrl(event.target.value)}
            placeholder="https://github.com/org/repo.git"
            value={repoUrl}
          />
          <label htmlFor="clone-destination-input">Destination folder</label>
          <div className="folder-picker-row">
            <input
              id="clone-destination-input"
              placeholder="Choose a destination folder from Finder"
              readOnly
              value={destinationRoot}
            />
            <button disabled={busyAction !== null} onClick={handleChooseCloneDestination} type="button">
              Choose destination…
            </button>
          </div>
          <button disabled={busyAction !== null || !repoUrl.trim() || !destinationRoot.trim()} onClick={handleCloneRepository} type="button">
            {busyAction === "clone" ? "Cloning…" : "Clone repo"}
          </button>
        </section>
      </div>

      <section className="repository-list-panel" aria-label="Repository list">
        <div className={isRepositorySearchOpen ? "repository-list-heading repository-list-heading--searching" : "repository-list-heading"}>
          <div className="repository-list-title-row">
            <h3>Repository list</h3>
            <div className={isRepositorySearchOpen ? "repository-search-morph repository-search-morph--open" : "repository-search-morph"}>
              {isRepositorySearchOpen ? (
                <label className="repository-search-field" htmlFor="repository-search-input">
                  <Search aria-hidden="true" size={16} strokeWidth={2.4} />
                  <input
                    autoFocus
                    id="repository-search-input"
                    onChange={(event) => setRepositorySearchQuery(event.target.value)}
                    placeholder="Search repositories…"
                    type="search"
                    value={repositorySearchQuery}
                  />
                  <button
                    aria-label="Close repository search"
                    className="repository-search-close"
                    onClick={() => {
                      setIsRepositorySearchOpen(false);
                      setRepositorySearchQuery("");
                    }}
                    type="button"
                  >
                    <X aria-hidden="true" size={15} strokeWidth={2.6} />
                  </button>
                </label>
              ) : (
                <button
                  aria-label="Search repositories"
                  className="repository-search-toggle"
                  onClick={() => setIsRepositorySearchOpen(true)}
                  type="button"
                >
                  <Search aria-hidden="true" size={17} strokeWidth={2.6} />
                </button>
              )}
            </div>
            <span>{repositoryCountLabel}</span>
          </div>
        </div>

        {repositoryScanFeedback ? (
          <div className={`repo-scan-feedback repo-scan-feedback--${repositoryScanFeedback.tone}`} role="status" aria-live="polite">
            <span>{repositoryScanFeedback.label}</span>
            <p>{repositoryScanFeedback.message}</p>
          </div>
        ) : null}

        {(defaultBranchStatus || defaultBranchError) ? (
          <div className={defaultBranchError ? "default-branch-feedback default-branch-feedback--error" : "default-branch-feedback"} role="status" aria-live="polite">
            <span>Default branch</span>
            <p>{defaultBranchError || defaultBranchStatus}</p>
          </div>
        ) : null}

        {repositories.length === 0 ? (
          <p className="repo-empty-state">No repositories added yet. Scan a folder or clone a GitHub repo link to populate this list.</p>
        ) : filteredRepositories.length === 0 ? (
          <p className="repo-empty-state">No repositories match “{repositorySearchQuery.trim()}”.</p>
        ) : (
          <div className="repository-card-list">
            {filteredRepositories.map((repository) => (
              <article className={recentlyAddedRepositoryIds.includes(repository.id) ? "repository-card repository-card--just-added" : "repository-card"} key={repository.id}>
                <div className="repository-card-heading">
                  <div>
                    <h4>{repository.name}</h4>
                    <span className={repository.dirty ? "repo-status repo-status--dirty" : "repo-status repo-status--clean"}>
                      {repository.dirty ? "Dirty" : "Clean"}
                    </span>
                  </div>
                  <button
                    aria-pressed={repository.id === linkedRepositoryId}
                    onClick={() => onLinkedRepositoryIdChange(repository.id)}
                    type="button"
                  >
                    {repository.id === linkedRepositoryId ? "Using for Agents" : "Use for Agents"}
                  </button>
                </div>
                <RepositoryMeta
                  branchOptions={editingDefaultBranchRepositoryId === repository.id ? defaultBranchOptions : []}
                  isEditingDefaultBranch={editingDefaultBranchRepositoryId === repository.id}
                  isUpdatingDefaultBranch={updatingDefaultBranchRepositoryId === repository.id}
                  onCancelDefaultBranchEdit={handleCancelDefaultBranchEdit}
                  onEditDefaultBranch={handleEditDefaultBranch}
                  onSaveDefaultBranch={handleSaveDefaultBranch}
                  onSelectedDefaultBranchChange={setSelectedDefaultBranch}
                  repository={repository}
                  selectedDefaultBranch={editingDefaultBranchRepositoryId === repository.id ? selectedDefaultBranch : ""}
                />
              </article>
            ))}
          </div>
        )}
      </section>
    </section>
  );
}
