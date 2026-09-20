# Bridge match-launch argv

Locked contract with Dock / `brawlback-launcher` (`matchLaunch.ts`).

Flags are appended after Dolphin's usual `-b -e <iso>` (ISO path stays user-configured; no Ladder HTTP here).

| Flag | Value |
|------|--------|
| `--bb-match-id` | string match id |
| `--bb-seed` | int64 decimal string (safe past JS `Number.MAX_SAFE_INTEGER`) |
| `--bb-local-idx` | int local player index into `--bb-endpoints` |
| `--bb-endpoints` | compact JSON `[{"playerId","host","port"},...]` |

There is **no** `--bb-host` / `isHost` flag. Listen role:

- Bind ENet on `endpoints[localIdx].port` (`ENET_HOST_ANY`).
- Connect outbound to every other endpoint.
- Existing netplay “host” (seed broadcast / `ProcessGameSettings`) is derived as `localIdx == 0`.

## Implementation

- Parse: `UICommon/CommandLineParse` → `Core/Brawlback/BridgeLaunchArgs`
- Connect: `CEXIBrawlback::handleFindMatch` bypasses Lylat when `BridgeLaunchArgs::IsActive()`, then `connectViaBridge()`
- Seed / idx applied in `handleStartMatch` (`GameSettings::randomSeed` takes low 32 bits of the int64)

## Failure modes

All paths fail closed: `IsActive()` stays false (or connect returns without starting Lylat).

| Condition | Behavior |
|-----------|----------|
| Partial `--bb-*` (not all four) | `SetFromCli` fails; `ERROR_LOG` in CLI parse; `IsActive()` false → normal Lylat path |
| Invalid `--bb-endpoints` JSON / non-array / missing fields | `SetFromCli` fails; `ERROR_LOG`; inactive |
| Empty `--bb-endpoints` `[]` | Rejected; inactive |
| Endpoint `port` missing / `0` / `>65535` | Rejected as missing or out of range; inactive |
| `--bb-local-idx` out of range | Rejected; inactive |
| Invalid `--bb-seed` / `--bb-local-idx` parse | Rejected; inactive |
| `IsActive()` true but `connectViaBridge` fails (no local endpoint, `enet_host_create`, no usable remotes) | `ERROR_LOG`; **no Lylat fallback**; ENet host destroyed on peer-connect failure |

`IsActive()` is true only after a successful `SetFromCli` with all four flags.

## Smoke-test example

```bash
dolphin-emu -b -e /path/to/game.iso \
  --bb-match-id match-smoke \
  --bb-seed 123456789012345 \
  --bb-local-idx 0 \
  --bb-endpoints '[{"playerId":"p0","host":"127.0.0.1","port":4096},{"playerId":"p1","host":"127.0.0.1","port":4097}]'
```

Peer process uses `--bb-local-idx 1` with the same endpoints / seed / match-id.

Unit tests: `BridgeLaunchArgsTest` (endpoints JSON + `SetFromCli`).
