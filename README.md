# Zoid

A local-first macOS operating desk for AI-assisted MaVoid work — agents, code, content, and automations in one native window.

Built for a founder or operator who needs a command room that stays readable, auditable, and fail-closed. Local runtime is the source of truth; the UI does not claim success until a provider or file read-back says so.

- Run Hermes agent sessions with files, queued work, and session focus
- Connect local repositories to implementation work with dirty-state awareness
- Operate content and social publishing only after provider proof
- Supervise local automations without leaking secrets into the UI
- Keep settings, models, and credentials on the machine

## Try it

```bash
npm install
npm run tauri:dev
```

Browser preview (no native bridge):

```bash
npm run dev
```

Zoid 25 is a Tauri + React + TypeScript desktop app (com.mavoid.zoid25). Product intent is in [PRODUCT.md](PRODUCT.md). Dev commands, local verification, and PR gates live in [Docs/development.md](Docs/development.md).

---

Built by [Ziad Ahmed](https://github.com/Ziad-NasrEldin) at [MaVoid](https://mavoid.com).

[Website](https://mavoid.com) · [LinkedIn](https://linkedin.com/in/ziad-ahmed-634202332) · [GitHub](https://github.com/Ziad-NasrEldin)
