# FrameLogHarness

Standalone model of Brawlback `CEXIBrawlback` predict / `updateSync` / `shouldRollback` (branch `savestates-efficiency-v2`), plus fixture checks against Bridge NDJSON (`Tools/FrameLog.md`).

## Prove

```bash
cargo test
```

Covers:

1. `shouldRollback` only when local and remote are past `latestConfirmed`
2. Predict-miss → rollback range + `framesToAdvance` (e.g. 158→155 ⇒ 4)
3. Matching prediction advances confirmed with no rollback
4. Fixture `fixtures/predict_miss.ndjson` parses Bridge join keys on `type=sync`

## NDJSON join fields (with Bridge)

`local`, `confirmed`, `predicting`, `rb_start`, `rb_stop`, `checksum` (+ optional `sync_*` / `frames_to_advance`)

## Merge order

Prefer dolphin **#2 → #3 → #1** so emit + argv land before this harness.

## Not claimed

Full Dolphin/P+ determinism, IncrementalRB page rewind, or menu desync fixes. Drop a real `brawlback_frame_log.ndjson` beside the fixture when you have a session.
