# Hermes chat bilingual Japanese labels

## Scope
User asked to undo replacing the Hermes header `代理` with `Quiet ink workspace`, and asked to add Japanese translations beside each button so both English and Japanese are visible.

## Changes
- `src/agents/AgentsHermesScreen.tsx`
  - Restored the Hermes title micro-label to `<p className="kana-line">代理</p>`.
  - Repository control label now shows English and Japanese: `Repository` + `接続`.
  - Unlinked repository dropdown fallback now shows both: `Unlinked / 未接続`.
  - Files button now shows both: `Files` + `書類`.
- `src/App.css`
  - Restored `.agents-sumi-e .hermes-title-block .kana-line` styling.
  - Added `.control-label-jp` and `.button-label-jp` styling using the Japanese serif stack.
  - Expanded the compact topbar control sizing to fit bilingual labels cleanly: repo column `192-252px`, Files button `112px`, total topbar max `600px`.
- `src/scaffold.test.ts`
  - Updated guards for restored `代理`, bilingual repo/files text, and adjusted compact sizing.
  - Made the Code surface guard read operation labels from `repositoryOperations.ts`, where the button labels are defined.
- `src/sessionState.ts`
  - Aligned root session type with existing operation-session fields used by `App.tsx`, fixing release TypeScript build drift.

## Validation
- `npm run build` passed.
- `npm run test:frontend` passed.
- Browser computed proof on Agents page:
  - title text includes `代理` and `Hermes Agents`.
  - repo text: `REPOSITORY / 接続 / Unlinked / 未接続`.
  - files button text: `FILES / 書類`.
  - Japanese button font is source-han-serif-japanese stack.
  - repo geometry: `192×36`; Files geometry: `112×36`.

## Review focus
Confirm the user’s explicit request was met: header label restored, Japanese translation visible beside topbar button/control labels, both English and Japanese visible, no layout/test/build regression.
