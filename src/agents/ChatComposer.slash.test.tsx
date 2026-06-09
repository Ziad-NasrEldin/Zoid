import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

const composer = readFileSync(new URL("./ChatComposer.tsx", import.meta.url), "utf8");
const css = readFileSync(new URL("../App.css", import.meta.url), "utf8");

for (const requiredComposerSurface of [
  "inlineSlashSearch",
  "inlineSlashCommands",
  "inlineSlashOpen",
  "composer-slash-dropup",
  "composer-deep-panel--slash",
  "slashPanelMaxHeight",
  "--composer-slash-panel-max-height",
  "Available slash commands",
  "↑↓ navigate · Tab inserts",
  "role=\"listbox\"",
  "role=\"option\"",
  "setMenuOpen(false);",
  "setActivePanel(null);",
  "getInlineSlashSearch(value)",
  "if (/\\s/.test(commandDraft)) return null;",
  "event.key === \"Tab\" || event.key === \"Enter\"",
  "scrollIntoView({ block: \"nearest\" })",
  "aria-activedescendant",
  "onMouseEnter={() => setHighlightedSlashCommandIndex(index)}",
  "insertCommand(command, false)",
  "scrollIntoView({ block: \"nearest\" })",
  "inlineSlashOptionRefs.current[index] = element",
]) {
  assert.ok(composer.includes(requiredComposerSurface), `ChatComposer must support type-/ slash command drop-up: ${requiredComposerSurface}`);
}

for (const requiredStyleSurface of [
  ".composer-input-column { position: relative;",
  ".composer-slash-dropup { position: absolute;",
  "bottom: calc(100% + 10px)",
  "grid-template-rows: auto minmax(0, 1fr)",
  ".composer-slash-dropup-list { min-height: 0;",
  "overflow: auto",
  ".composer-deep-panel--slash { grid-template-rows: auto auto auto minmax(0, 1fr); max-height: var(--composer-slash-panel-max-height, min(560px, calc(100vh - 260px))); overflow: hidden; }",
  ".composer-deep-panel--slash .composer-panel-helper { max-height: 48px; overflow: auto; }",
  ".slash-command-list { grid-template-columns: repeat(2, minmax(0, 1fr)); min-height: 0; overflow: auto; padding-right: 2px; }",
  ".composer-slash-dropup-option--active",
]) {
  assert.ok(css.includes(requiredStyleSurface), `App.css must style the slash command drop-up: ${requiredStyleSurface}`);
}

assert.ok(!composer.includes("matches.slice(0, 9)"), "Slash command drop-up must show the full live Hermes registry and scroll instead of truncating commands");
