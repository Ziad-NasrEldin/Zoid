import asyncio
import json
import os
import re
from datetime import datetime
from pathlib import Path

import yaml
from mcp import ClientSession
from mcp.client.streamable_http import streamablehttp_client

HOME = Path.home()
HERMES = HOME / ".hermes"
MODULE = Path('/Users/ziadnasreldin/Zoid/Docs/modules/code-workspace')
PROMPT_PATH = MODULE / 'stitch-ai-screen-design-prompt.md'
RESULT_PATH = MODULE / 'stitch-generation-results.json'
SUMMARY_PATH = MODULE / 'stitch-generation-summary.md'


def load_env():
    env_path = HERMES / '.env'
    if not env_path.exists():
        return
    for line in env_path.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith('#') or '=' not in line:
            continue
        k, v = line.split('=', 1)
        os.environ.setdefault(k.strip(), v.strip().strip('"').strip("'"))


def expand(s: str) -> str:
    return re.sub(r'\$\{([^}]+)\}', lambda m: os.environ.get(m.group(1), ''), s)


def jsonable(obj):
    if hasattr(obj, 'model_dump'):
        return obj.model_dump(mode='json')
    if hasattr(obj, '__dict__'):
        return obj.__dict__
    return str(obj)


GLOBAL = """
Design this as part of the Zoid Code Workspace module. Visual style: native macOS, Apple Finder-style project browser plus operational health dashboard, clean compact rows, right inspector like Finder/Apple Settings, low chrome, hairline borders, status chips, Action Blue #0066cc, SF/Google Sans-like typography, light mode, not enterprise-table heavy. Product/client grouping first. Use realistic sample data for Zoid, MaVoid Site, Leadra, Kalima, and a disposable test repo. Designs must imply real local repos, real git state, real checks, evidence-based launch gates, and safe confirmation-gated git/deploy actions.
""".strip()

SCREENS = [
    ("01 Code Workspace — Repo Health Command Center", "Main landing screen with header, Search Repos, Add Repo, Scan Folders, Refresh Status, Code Settings, small summary counters, product/client grouped repo list, attention queue, active code agent sessions, right inspector, quick actions Open Repo, Start Agent, Run Checks, View Diff, Launch Gate, compact rows with optional card/grid toggle."),
    ("02 Repo Discovery / Scan Folders", "Add scan roots and approve discovered repos. Include scan source panel, Add Scan Folder, Run Scan, suggested repos list, bulk actions, repo preview inspector, Ignored tab with Restore, explicit warning that Zoid only scans user-added folders."),
    ("03 Managed Repositories List", "Approved repo browser with grouped rows, search/filter toolbar, columns/chips for name/path/profile/branch/dirty/checks/deployment/launch gate/active agent/activity, quick actions and right inspector."),
    ("04 Repository Detail", "Full repository detail page answering whether repo is safe, clean, and ready to work on or launch. Include overview, git status, changed files, diff, branches, commits, PRs, deployments, launch gate, linked items, history, settings, project notes, danger zone, stale warnings, compare current diff vs last verified launch."),
    ("05 Right Inspector", "Selected-repo inspector tabs: Summary, Diff, Checks, Agents, Launch, Deployments, Linked Items, History. Include no-selection, command output collapsed, evidence missing, agent waiting, deployment unverified states."),
    ("06 Repo Settings / Rules", "Repo profile/settings form with display name, product/client group, profile type, local path, remote URL, default branch, command overrides, deployment targets, launch checklist template, production URLs, agent permissions, sensitive file patterns, preferred agent/model, reviewer requirement, evidence storage, ignore/archive, notes, imported rules pending approval."),
    ("07 Run Checks Flow", "Explicit check runner with command checklist typecheck/lint/test/build/verify, command source labels, run selected/all, collapsed live output, pass/fail/duration/timestamp, save as launch evidence, failed check Start Agent action."),
    ("08 Start Agent From Repo Modal", "Modal to launch Agents Workspace session from repo with agent profile, model/provider, repo/workdir selected, diff/status/rules attached, suggested prompts, permission preview, layout choice, sensitive files excluded, linked run visible after start."),
    ("09 Launch Gate", "Evidence-based launch gate with header state/repo/product/task/commit/deployment, checklist git/typecheck/lint/tests/build/review/push/PR/deployment/prod E2E/backend API/database, evidence panel, verdict panel, history, actions, Mark Verified disabled until evidence exists, manual override sheet."),
    ("10 Evidence Attachment / Verification Record", "Evidence add/edit screen with type selector, required/supporting toggle, source types command output/diff/commit/review/PR/deployment/E2E/API/database/screenshot/video/log/manual note, attach file/link, timestamp/source metadata, storage explanation."),
    ("11 GitHub / PR Integration", "Remote integration screen with remote URL, connect GitHub unauthenticated state, global/per-repo auth indicators, open PRs, issue/task links, CI status, create PR draft flow with editable title/body and confirmation."),
    ("12 Deployment Tracking / Actions", "Deployment targets and explicit actions for Vercel, Hostinger VPS/SSH/Docker/Nginx, GitHub Pages, Cloudflare Pages/R2, custom/manual. Show environment, production URL, deploy command/provider, last deploy/verified status, E2E checklist, rollback notes, manual deployment record, confirmation."),
    ("13 Commit / Git Action Workflow", "Safe git workflow: status/diff/history/branches, stage selected files, commit editable generated message, create branch, stash, pull if clean, strong confirmation for push/merge/rebase/reset/dirty checkout/discard/delete/force push/rollback-linked commit, danger zone, protected branch warning."),
    ("14 Repo Handoff Export", "One-click repo state export with summary preview repo/branch/dirty/changed files/checks/active agents/launch gate/deployment/stale warnings/notes, include/exclude options, export clean summary, copy, attach to agent, save to repo/docs or app data."),
    ("15 Search / History / Archive", "Global search across repos by name/path/group/profile/status/file/PR/agent/evidence/history, filters, result rows with matched excerpt, archive/ignored management, history timeline of repo events/checks/agents/launch gates/deployments/commits/PRs/evidence."),
    ("16 Native Verification / Diagnostics", "Diagnostics screen proving native state: Tauri/native indicator, browser preview warning, scan root status, managed registry status, disposable test repo status, git availability, GitHub auth status, deployment provider availability, app data/evidence paths, verification checklist and test actions."),
]

async def call(session, name, args):
    res = await session.call_tool(name, args)
    return jsonable(res)

async def main():
    load_env()
    conf = yaml.safe_load((HERMES / 'config.yaml').read_text())
    st = conf['mcp_servers']['stitch']
    headers = {k: expand(str(v)) for k, v in st.get('headers', {}).items()}
    url = st['url']
    prompt_full = PROMPT_PATH.read_text()
    results = {"started_at": datetime.now().isoformat(), "project": None, "design_system": None, "screens": []}

    async with streamablehttp_client(url, headers=headers, timeout=600) as (read, write, _):
        async with ClientSession(read, write) as session:
            await session.initialize()
            project_res = await call(session, 'create_project', {"title": "Zoid Code Workspace Module"})
            results['project_create_result'] = project_res
            text = json.dumps(project_res)
            m = re.search(r'projects/(\d+)', text)
            if not m:
                m = re.search(r'projectId["\']?\s*[:=]\s*["\']?(\d+)', text)
            if not m:
                raise RuntimeError(f"Could not extract project id from create_project result: {project_res}")
            project_id = m.group(1)
            results['project_id'] = project_id

            ds_args = {
                "projectId": project_id,
                "designSystem": {
                    "displayName": "Zoid Apple Native Light",
                    "theme": {
                        "colorMode": "LIGHT",
                        "headlineFont": "GOOGLE_SANS",
                        "bodyFont": "GOOGLE_SANS_TEXT",
                        "labelFont": "GOOGLE_SANS_TEXT",
                        "roundness": "ROUND_TWELVE",
                        "customColor": "#0066cc",
                        "colorVariant": "NEUTRAL",
                        "overridePrimaryColor": "#0066cc",
                        "overrideNeutralColor": "#f5f5f7",
                        "overrideSecondaryColor": "#34c759",
                        "overrideTertiaryColor": "#ff9500",
                        "designMd": "Zoid design system: native macOS, Apple-inspired, light mode, low chrome, compact rows, right inspector, Action Blue #0066cc, hairline borders, status chips, rounded 12px surfaces, readable operational UI."
                    }
                }
            }
            ds_res = await call(session, 'create_design_system', ds_args)
            results['design_system_create_result'] = ds_res
            ds_text = json.dumps(ds_res)
            dm = re.search(r'assets/(\d+)', ds_text)
            design_system = f"assets/{dm.group(1)}" if dm else None
            results['design_system'] = design_system

            for idx, (title, brief) in enumerate(SCREENS, 1):
                prompt = f"""{GLOBAL}

Screen title: {title}

Screen-specific requirements:
{brief}

Use the full module brief below as source of truth. Create one high-fidelity desktop macOS app screen for this screen only. Include realistic data, strong hierarchy, right inspector/panels where relevant, empty/loading/error/blocked/success state cues where appropriate, and clear primary/secondary actions.

Full module brief:
{prompt_full[:12000]}
"""
                args = {
                    "projectId": project_id,
                    "prompt": prompt,
                    "deviceType": "DESKTOP",
                    "modelId": "GEMINI_3_1_PRO",
                }
                if design_system:
                    args["designSystem"] = design_system
                try:
                    res = await call(session, 'generate_screen_from_text', args)
                    results['screens'].append({"index": idx, "title": title, "status": "ok", "result": res})
                except Exception as e:
                    results['screens'].append({"index": idx, "title": title, "status": "error", "error": f"{type(e).__name__}: {e}"})
                RESULT_PATH.write_text(json.dumps(results, indent=2), encoding='utf-8')

            try:
                screens = await call(session, 'list_screens', {"projectId": project_id})
                results['list_screens_result'] = screens
            except Exception as e:
                results['list_screens_error'] = f"{type(e).__name__}: {e}"

    results['finished_at'] = datetime.now().isoformat()
    RESULT_PATH.write_text(json.dumps(results, indent=2), encoding='utf-8')

    ok = [s for s in results['screens'] if s['status'] == 'ok']
    bad = [s for s in results['screens'] if s['status'] != 'ok']
    project_url = f"https://stitch.withgoogle.com/projects/{results['project_id']}"
    lines = [
        "# Stitch Generation Summary: Code Workspace",
        "",
        f"Generated at: {results['finished_at']}",
        f"Project ID: `{results['project_id']}`",
        f"Project URL: {project_url}",
        f"Design system: `{results.get('design_system')}`",
        f"Screens requested: {len(SCREENS)}",
        f"Screens generated successfully: {len(ok)}",
        f"Screens failed: {len(bad)}",
        "",
        "## Screen Results",
    ]
    for s in results['screens']:
        lines.append(f"- {s['index']:02d}. {s['title']} — {s['status']}")
        if s['status'] != 'ok':
            lines.append(f"  - Error: {s['error']}")
    lines += ["", "## Raw Results", f"See `{RESULT_PATH}`"]
    SUMMARY_PATH.write_text("\n".join(lines) + "\n", encoding='utf-8')
    print("PROJECT_ID", results['project_id'])
    print("PROJECT_URL", project_url)
    print("SUMMARY", SUMMARY_PATH)
    print("RESULTS", RESULT_PATH)
    print("OK", len(ok), "FAILED", len(bad))

if __name__ == '__main__':
    asyncio.run(main())
