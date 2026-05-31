# Spike A: PTY / CLI Runtime Feasibility

Date: 2026-05-31
Plan: `/Users/ziadnasreldin/Zoid/Docs/2026-05-31-zoid-implementation-plan-v1.md`
Result: Partial

## Goal

Prove Zoid can run real local CLI/session workflows reliably before committing to the main app architecture.

## Summary

Spike A partially passed using a runtime prototype outside the repo. The prototype proved shell command execution, interactive PTY execution, incremental stdout/stderr streaming, stdin, cancellation by process group, app-support log persistence, SQLite metadata, exit-code/duration tracking, log rotation, and basic secret redaction.

It did not prove a real prototype UI. The Clean Session UI/card rendering acceptance is therefore only partially satisfied by a text/card parser over streamed logs, not by an actual Tauri/React UI. This must be validated in the first real app-shell implementation.

## Prototype Artifacts

Temporary script:

```text
/tmp/zoid_spike_a.py
```

App support output:

```text
/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeA/
```

SQLite metadata DB:

```text
/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeA/spike_a_sessions.sqlite3
```

Log files:

```text
/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeA/*.ndjson
```

## Commands Run

```bash
python3 /tmp/zoid_spike_a.py /Users/ziadnasreldin/Zoid
```

Additional verification:

```bash
python3 - <<'PY'
import sqlite3
p='/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeA/spike_a_sessions.sqlite3'
c=sqlite3.connect(p)
for r in c.execute('select id,kind,cwd,exit_code,state,round(duration,3),log_ref from sessions order by started desc limit 3'):
    print(r)
PY
```

```bash
python3 - <<'PY'
from pathlib import Path
base=Path('/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeA')
for pat in ['shell-1780249118027.ndjson','pty-1780249118669.ndjson','cancel-1780249118896.ndjson']:
    p=base/pat
    print('\n--',pat,p.stat().st_size,'bytes')
    for line in p.read_text().splitlines()[:4]: print(line[:300])
PY
```

## Verified Output Summary

Main run output included:

```text
APP_SUPPORT=/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeA
DB=/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeA/spike_a_sessions.sqlite3

SHELL id=shell-1780249118027 exit=3 duration=0.636 log=/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeA/shell-1780249118027.ndjson events=7
first={'t': 0.005, 'stream': 'stdout', 'data': '/Users/ziadnasreldin/Zoid\nOUT1\n'}
last={'t': 0.635, 'stream': 'stdout', 'data': 'token=[REDACTED]\n'}

PTY id=pty-1780249118669 exit=7 duration=0.223 log=/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeA/pty-1780249118669.ndjson events=2
cards=[{'type': 'status', 'status': 'running', 'message': 'hello-from-pty\\n"'}, {'type': 'status', 'status': 'running', 'message': 'hello-from-pty'}]

CANCEL id=cancel-1780249118896 exit=-15 duration=1.058 log=/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeA/cancel-1780249118896.ndjson childpid=5462 child_alive_after_kill=False kill_method=process-group events=2

ROTATION id=rotation-1780249119958 files=[
  'rotation-1780249119958.ndjson.1:1978',
  'rotation-1780249119958.ndjson.2:1988',
  'rotation-1780249119958.ndjson.3:1988',
  'rotation-1780249119958.ndjson.4:1988',
  'rotation-1780249119958.ndjson.5:1988',
  'rotation-1780249119958.ndjson:1420'
]

REDACTION leaks=[]
```

SQLite verification output:

```text
('cancel-1780249118896', 'cancel', '/Users/ziadnasreldin/Zoid', -15, 'failed', 1.058, '/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeA/cancel-1780249118896.ndjson')
('pty-1780249118669', 'pty', '/Users/ziadnasreldin/Zoid', 7, 'failed', 0.223, '/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeA/pty-1780249118669.ndjson')
('shell-1780249118027', 'shell', '/Users/ziadnasreldin/Zoid', 3, 'failed', 0.636, '/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeA/shell-1780249118027.ndjson')
```

## Requirement Findings

| Requirement | Result | Evidence / Notes |
|---|---:|---|
| Start shell command in chosen cwd | Pass | Shell printed `/Users/ziadnasreldin/Zoid`. |
| Start interactive PTY command | Pass | `/bin/bash --noprofile --norc -i` ran through PTY. |
| Stream stdout/stderr incrementally | Pass | Selector loop captured timestamped stdout/stderr events separately. |
| Send stdin to running process | Pass | PTY received `pwd`, card output, secret echo, and `exit 7`. |
| Cancel/kill process tree | Pass | `start_new_session` + `os.killpg`; child was not alive after kill. |
| Persist raw logs to app support | Pass | NDJSON logs stored under Application Support, outside SQLite. |
| Render clean output cards | Partial | Prototype parsed `CARD:status=running`; no actual Tauri/React prototype UI was built. PTY echo caused one noisy duplicate candidate. |
| Reopen/show previous session history | Pass | SQLite metadata query returned prior shell/pty/cancel sessions. |
| Detect exit code/duration/failure | Pass | exit `3`, `7`, and `-15` recorded with durations. |
| Enforce max log size/rotation | Pass | Prototype rotated 2 KB log files into `.ndjson.1` through `.5`. |
| Redact obvious secrets | Pass | `token=sk...` and `api_key=ghp...` became `[REDACTED]`; `leaks=[]`. |
| Test command and interactive command run from prototype UI | Not verified | Commands ran from runtime prototype, not from a UI. This remains a required first app-shell validation. |

## Fallbacks / Mitigations

| Risk / Unsupported capability | Recommended fallback / mitigation |
|---|---|
| No real prototype UI yet | Build the first Tauri shell around the PTY runner before broad app work; keep Spike A as runtime-only evidence until UI streaming is proven. |
| PTY echo, prompts, CRLF/control noise | Implement a terminal-output normalizer and structured card protocol; keep raw logs collapsible and never rely on raw PTY text for primary UX. |
| Detached daemonized grandchildren may survive process-group kill | Add process-supervision policy: prefer child process groups, track spawned PIDs where possible, expose “possibly still running” warning, and avoid treating kill as verified unless process tree check passes. |
| Regex-only redaction misses unknown secret formats | Centralize redaction service with test corpus; allow integration-specific secret patterns; run redaction before persistence and before rendering. |
| Log rotation lacks retention/atomicity proof | Implement append-with-temp/rename or durable file handling, max file count, retention policy, and tests for truncation/rotation under concurrent stream writes. |

## Unsupported / Risks

- Prototype was Python/POSIX, not final Tauri/Rust UI code.
- PTY logs contain echo, prompts, CRLF/control noise; production renderer needs parsing/deduping.
- Process tree kill works for children in the new process group; detached daemonized grandchildren need stronger supervision.
- Redaction was regex-based for obvious secrets only; production must centralize broader redaction tests.
- Log rotation needs retention cleanup and atomic write behavior.

## Stack Decision Impact

- No runtime blocker found for Tauri + native Rust layer.
- The required macOS primitives are feasible.
- Node helper is not proven necessary for CLI runtime; keep it optional unless Rust/Tauri PTY implementation fails.
- Proceed with Tauri preference, but implement the production PTY layer early and test against real Hermes-style long-running CLI sessions.
