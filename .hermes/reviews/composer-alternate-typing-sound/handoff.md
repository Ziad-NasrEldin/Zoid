# Handoff: alternate composer typing sound

## Scope

User asked to try a different composer typing sound in Zoid 25.

## Implementation

Changed `/Users/ziadnasreldin/Zoid/src/agents/ChatComposer.tsx` Web Audio typing feedback from the previous lower, warmer lowpass pulse to a lighter short click/chime:

- `TYPING_SOUND_MIN_INTERVAL_MS`: `70`
- `TYPING_SOUND_VOLUME`: `0.009`
- New `TYPING_SOUND_CLICK_VOLUME`: `0.0035`
- Primary oscillator: `sine`, ~640–710 Hz for inserts, 420 Hz for deletes
- Secondary click oscillator: `triangle`, brief higher overtone
- Filter: `bandpass` at 780 Hz, Q 0.72
- Shorter envelope: primary stops at 0.075s, click stops at 0.04s
- Still skips history/paste input and throttles per-key playback.

Updated `/Users/ziadnasreldin/Zoid/src/scaffold.test.ts` guard strings to require the new sound shape (`TYPING_SOUND_CLICK_VOLUME`, sine + triangle, `bandpass`).

## Verification run

- `npm run test:frontend && npm run build`: PASS
- `npm run tauri:build`: PASS
- Reinstalled bundle to `/Applications/Zoid 25.app`
- Relaunched with `/Applications/Zoid 25.app/Contents/MacOS/zoid`
- Process verification: app is running as PID 28850.

## Review focus

Please review only the alternate typing sound change and guard update. Check for regressions such as too-loud volume, missing cleanup/disconnect, TypeScript issues, browser/WebKit AudioContext compatibility, or accidental paste/history playback.
