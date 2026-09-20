# Bridge in-emu frame log (NDJSON)

Spike helper: `Source/Core/Core/Brawlback/FrameLog.h`

Writes `brawlback_frame_log.ndjson` next to the Dolphin executable (`File::GetExeDirectory()`).

## Gating

- Compile: `-DBRAWLBACK_FRAME_LOG=0` disables.
- Runtime: `BRAWLBACK_FRAME_LOG=0` disables.
- Spike default: **on**.

## Join keys (match Rollback `Tools/FrameLogHarness`)

| Key | Source (`CEXIBrawlback`) |
|-----|--------------------------|
| `local` | local frame (`locFrame` after `updateSync`) |
| `confirmed` | `latestConfirmedFrame` |
| `predicting` | `isPredicting` |
| `rb_start` | `startRollbackFrame` (null if no rollback this sync) |
| `rb_stop` | `stopRollbackFrame` (null if no rollback this sync) |
| `checksum` | savestate checksum when available; else `-1` |

Optional `SyncData` (from local `PlayerFrameData` when present):

`sync_locX`, `sync_locY`, `sync_anim`, `sync_percent`, `sync_stocks`, `sync_facing`

## `type` values

- `pad` — after `storeLocalInputs` (`local`, `playerIdx`, `buttons`)
- `advance` — `handleFrameAdvanceRequest` (`frames_to_advance`)
- `sync` — end of `updateSync` (join keys above)

JSON discriminator field is `type` (not `event`).

Does **not** rewrite Rollback's harness; emit matching keys from the C++ EXI path only.
