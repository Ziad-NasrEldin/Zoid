# Feature Handoff: Zoid 25 clean macOS scaffold

## Original request

"i want oyu to clean up all the UI already created in zoid ai os, clean up everything, i dont want any frontend at all, just navigation side bar and thats it
i will restart the whole project, i want you to keep all the old files in a separate directory and organize it please so we can easily get back to it later
organize all the folders, sub folders and inner files and everything please
all i want now is a completely new and clean macos desktop app setup, just a basic scaffold to start working on, the new one called zoid 25"

## Implementation summary

- Archived old Zoid AI OS UI/project work into `legacy/zoid-ai-os-ui-archive-2026-06-06/` with grouped folders for frontend, native source, docs, Hermes review artifacts, tooling, and data backups.
- Replaced the live React app with a minimal Zoid 25 desktop scaffold: macOS-style sidebar navigation plus blank canvas only.
- Removed the prior dev toolbar injection from `index.html` and removed the `agentation` dependency.
- Simplified frontend tests to a scaffold smoke test.
- Simplified Tauri native source to a minimal app runner and renamed product metadata to Zoid 25.

## Changed files

- `src/App.tsx`: new sidebar-only Zoid 25 scaffold.
- `src/App.css`: new minimal macOS-style sidebar/canvas styling.
- `src/main.tsx`: minimal React mount, no dev toolbar.
- `src/scaffold.test.ts`: smoke checks for Zoid 25 brand, primary navigation, blank canvas.
- `src/vite-env.d.ts`: Vite type reference restored for clean source tree.
- `index.html`: title changed to Zoid 25; removed old live tooling script injection.
- `package.json`: renamed to `zoid-25`, updated scripts, removed `agentation`.
- `package-lock.json`: regenerated after dependency cleanup.
- `src-tauri/tauri.conf.json`: product/window/bundle metadata changed to Zoid 25.
- `src-tauri/Cargo.toml`: version/description updated and unused old backend dependencies removed.
- `src-tauri/src/lib.rs`: minimal Tauri runner.
- `src-tauri/src/main.rs`: minimal app entry.
- `legacy/zoid-ai-os-ui-archive-2026-06-06/README.md`: archive map and restore guide, including previous migrations.
- `legacy/zoid-ai-os-ui-archive-2026-06-06/**`: archived old frontend/native/docs/reviews/tooling/data files, including previous `src-tauri/migrations/`.

## How to test

- `npm run test`
- `npm run build`
- `npm run tauri:build`
- Launch packaged app: `open -n "src-tauri/target/release/bundle/macos/Zoid 25.app"` and confirm a `Contents/MacOS/zoid` process starts.

## Tests run

- `npm install`: PASS, removed 1 package, 0 vulnerabilities.
- `npm run test`: PASS, frontend scaffold smoke and Rust tests passed after migration archive fix.
- `npm run build`: PASS, TypeScript + Vite production build passed after migration archive fix.
- `npm run tauri:build`: PASS, built release binary, `.app`, and `.dmg` after migration archive fix.
- `open -n "src-tauri/target/release/bundle/macos/Zoid 25.app" && sleep 3 && pgrep -fl "/Zoid 25.app/Contents/MacOS/zoid"`: PASS, showed running packaged app process ID 70997. Process was then stopped with `pkill`.

## Git info

- Branch: `main`
- Commit SHA, if committed: not committed
- Diff base, if known: working tree against current `main`

## Frontend/backend/database notes

- Frontend routes/components: single React app entry with sidebar-only scaffold and blank canvas.
- Backend endpoints/services: none; Tauri shell only.
- Database tables/migrations: none in the new scaffold. Previous SQLite migration SQL files are archived under `legacy/zoid-ai-os-ui-archive-2026-06-06/native/migrations/`.

## Reviewer focus areas

- Confirm the live source tree no longer imports old UI/workspace modules.
- Confirm old files are recoverable from the organized archive.
- Confirm app is renamed Zoid 25 in web title and Tauri metadata.
- Confirm verification commands are sufficient for a clean macOS desktop scaffold.

## Fix cycle notes

Fixed R1 from first critique: moved live `src-tauri/migrations/` into `legacy/zoid-ai-os-ui-archive-2026-06-06/native/migrations/`, updated the archive README, then re-ran `npm run test`, `npm run build`, `npm run tauri:build`, and packaged app launch/process verification successfully.
