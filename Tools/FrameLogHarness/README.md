# FrameLogHarness

Standalone model of Brawlback `CEXIBrawlback` predict / `updateSync` / `shouldRollback` (branch `savestates-efficiency-v2`).

## Prove

```bash
cargo test
```

Covers:

1. `shouldRollback` only when local and remote are past `latestConfirmed`
2. Predict-miss → rollback range + `framesToAdvance` (e.g. 158→155 ⇒ 4)
3. Matching prediction advances confirmed with no rollback

## NDJSON join fields (with Bridge)

`local`, `confirmed`, `predicting`, `rb_start`, `rb_stop`, `checksum` (+ optional `sync_*` from `SyncData`)

## Not claimed

Full Dolphin/P+ determinism, IncrementalRB page rewind, or menu desync fixes. Those need Bridge’s in-emu NDJSON emit + a real session.
