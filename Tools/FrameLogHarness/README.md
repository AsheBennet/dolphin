# FrameLogHarness

Standalone model of Brawlback `CEXIBrawlback` predict / `updateSync` / `shouldRollback`, plus Bridge NDJSON fixtures (no ISO).

## Prove

```bash
cargo test
```

Includes dual-peer desync: `first_desync` on `fixtures/peer_a.ndjson` vs `peer_b.ndjson` reports first `local` where `checksum` / `sync_*` diverge.

## NDJSON join fields

`local`, `confirmed`, `predicting`, `rb_start`, `rb_stop`, `checksum`, optional `sync_*` / `frames_to_advance`
