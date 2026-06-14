import { strict as assert } from "node:assert";
import { Window } from "happy-dom";
import { createRoot } from "react-dom/client";
import { CodeWorkspace } from "./CodeWorkspace";
import type { RepositoryOperationAction } from "./repositoryOperations";
import type { CodeRepository } from "./types";

const window = new Window({ url: "http://127.0.0.1:1420" }) as unknown as Window & typeof globalThis;
const document = window.document as Document;

Object.assign(globalThis, {
  window,
  document,
  HTMLElement: window.HTMLElement,
  HTMLButtonElement: window.HTMLButtonElement,
  HTMLInputElement: window.HTMLInputElement,
  HTMLTextAreaElement: window.HTMLTextAreaElement,
  Node: window.Node,
  MouseEvent: window.MouseEvent,
  Event: window.Event,
  localStorage: window.localStorage,
  requestAnimationFrame: window.requestAnimationFrame.bind(window),
  cancelAnimationFrame: window.cancelAnimationFrame.bind(window),
});

window.localStorage.clear();

async function waitForReact() {
  await new Promise((resolve) => window.setTimeout(resolve, 0));
}

const repository: CodeRepository = {
  id: "repo-1",
  name: "Critical Prod Repo",
  path: "/Users/ziadnasreldin/Critical Prod Repo",
  remoteUrl: "git@github.com:mavoid/critical.git",
  branch: "main",
  defaultBranch: "main",
  dirty: true,
  addedAt: "2026-06-09T00:00:00.000Z",
  source: "scanned",
};

const anotherRepository: CodeRepository = {
  ...repository,
  id: "repo-2",
  name: "Another Repo",
  path: "/Users/ziadnasreldin/Another Repo",
  remoteUrl: "git@github.com:mavoid/another.git",
  dirty: false,
};

async function click(button: HTMLButtonElement) {
  button.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }) as unknown as Event);
  await waitForReact();
}

async function renderCodeWorkspace(onRepositoryOperationStart: (repo: CodeRepository, action: RepositoryOperationAction) => void) {
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const root = createRoot(container);
  root.render(
    <CodeWorkspace
      onRepositoriesChange={() => undefined}
      onRepositoryOperationStart={onRepositoryOperationStart}
      repositories={[repository, anotherRepository]}
    />,
  );
  await waitForReact();
  return { container, root };
}

const startedActions: RepositoryOperationAction[] = [];
const { container, root } = await renderCodeWorkspace((_repo, action) => startedActions.push(action));

assert.equal(container.textContent?.includes("Use for Agents"), false, "Code workspace must not advertise a global Agents repository link action");
assert.equal(container.textContent?.includes("Using for Agents"), false, "Code workspace must not imply a repository is globally linked for Agents");
assert.equal(container.textContent?.includes("Selected for Agents"), false, "Code workspace must not imply a repository is selected for Agents");
assert.equal(container.querySelector(".repository-link-button"), null, "Code workspace must not render a repository-to-Agents link button");

const localhostButton = [...container.querySelectorAll<HTMLButtonElement>("button")].find((button) => button.textContent?.includes("Run localhost"));
assert.ok(localhostButton, "localhost action button should render");
await click(localhostButton);
assert.deepEqual(startedActions, ["localhost"], "localhost should start immediately without confirmation");

let confirmCalls = 0;
window.confirm = (message?: string) => {
  confirmCalls += 1;
  assert.match(String(message), /Critical Prod Repo/, "production confirmation should name the repository");
  assert.match(String(message), /Dirty state: Dirty/, "production confirmation should expose dirty state");
  assert.match(String(message), /irreversible deploys/, "production confirmation should explain irreversible risk");
  return false;
};
const productionButton = [...container.querySelectorAll<HTMLButtonElement>("button")].find((button) => button.textContent?.includes("Deploy production"));
assert.ok(productionButton, "production action button should render");
await click(productionButton);
assert.equal(confirmCalls, 1, "production should require app-level confirmation");
assert.deepEqual(startedActions, ["localhost"], "cancelled production confirmation must not start the operation");

window.confirm = () => true;
await click(productionButton);
assert.deepEqual(startedActions, ["localhost", "production"], "confirmed production operation should start exactly once");

root.unmount();
await waitForReact();
console.log("CodeWorkspace behavior tests passed");
