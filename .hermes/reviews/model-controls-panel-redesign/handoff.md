# Model controls panel redesign handoff

Feature slug: model-controls-panel-redesign

Scope:
- Replaced the generic model command panel styling with a purpose-built runtime sheet.
- Added dark ink current-state column for Provider / Model / Reasoning.
- Converted controls to stacked full-width rows instead of cramped three-column cards.
- Removed the generic footer sentence from the model panel instance.
- Updated scaffold guards to enforce the redesigned layout.

Changed files:
- src/agents/AgentsHermesScreen.tsx
- src/App.css
- src/scaffold.test.ts

Validation already run:
- npx tsx src/scaffold.test.ts
- npm run build
- npm run test:frontend
- npm run tauri:build

Known issue during visual verification:
- The Page Feedback overlay and macOS spaces interfered with repeated coordinate screenshots. The Tauri bundle was rebuilt and relaunched; dist asset contains the new dark-left-column CSS.
