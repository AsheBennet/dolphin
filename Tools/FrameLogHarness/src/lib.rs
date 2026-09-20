//! Standalone model of Brawlback's GGPO-shaped updateSync / shouldRollback /
//! getRemoteInputs predict path (AsheBennet/dolphin savestates-efficiency-v2).
//! Also parses Bridge `brawlback_frame_log.ndjson` fixtures (no ISO).

pub const MAX_ROLLBACK_FRAMES: u32 = 5;
pub const FRAME_DELAY: u32 = 1;
pub const GAME_FULL_START_FRAME: u32 = 150;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pad {
    pub buttons: u16,
    pub stick_x: i8,
    pub stick_y: i8,
}

impl Pad {
    pub fn blank() -> Self {
        Self {
            buttons: 0,
            stick_x: 0,
            stick_y: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerFrame {
    pub frame: u32,
    pub player_idx: u8,
    pub pad: Pad,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameLogEntry {
    pub local_frame: u32,
    pub latest_confirmed: u32,
    pub predicting: bool,
    pub rollback_start: Option<u32>,
    pub rollback_stop: Option<u32>,
    pub frames_to_advance: u32,
    pub sync_checksum: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NdjsonSync {
    pub local: u32,
    pub confirmed: u32,
    pub predicting: bool,
    pub rb_start: Option<u32>,
    pub rb_stop: Option<u32>,
    pub checksum: i64,
    pub frames_to_advance: Option<u32>,
    pub sync_percent: Option<f32>,
    pub sync_stocks: Option<u8>,
    pub sync_loc_x: Option<f32>,
    pub sync_loc_y: Option<f32>,
    pub sync_anim: Option<u32>,
    pub sync_facing: Option<i8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesyncDiverge {
    pub local: u32,
    pub reasons: Vec<&'static str>,
}

/// Minimal NDJSON parse for Bridge `type=sync` lines (join keys only).
pub fn parse_sync_events(ndjson: &str) -> Result<Vec<NdjsonSync>, String> {
    let mut out = Vec::new();
    for (lineno, line) in ndjson.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !line.contains("\"type\":\"sync\"") && !line.contains("\"type\": \"sync\"") {
            continue;
        }
        out.push(parse_sync_line(line).map_err(|e| format!("line {}: {}", lineno + 1, e))?);
    }
    Ok(out)
}

fn parse_sync_line(line: &str) -> Result<NdjsonSync, String> {
    Ok(NdjsonSync {
        local: json_u32(line, "local")?,
        confirmed: json_u32(line, "confirmed")?,
        predicting: json_bool(line, "predicting")?,
        rb_start: json_opt_u32(line, "rb_start")?,
        rb_stop: json_opt_u32(line, "rb_stop")?,
        checksum: json_i64(line, "checksum")?,
        frames_to_advance: json_opt_u32(line, "frames_to_advance").ok().flatten(),
        sync_percent: json_opt_f32(line, "sync_percent").ok().flatten(),
        sync_stocks: json_opt_u8(line, "sync_stocks").ok().flatten(),
        sync_loc_x: json_opt_f32(line, "sync_locX").ok().flatten(),
        sync_loc_y: json_opt_f32(line, "sync_locY").ok().flatten(),
        sync_anim: json_opt_u32(line, "sync_anim").ok().flatten(),
        sync_facing: json_opt_i8(line, "sync_facing").ok().flatten(),
    })
}

/// Compare two peers' `type=sync` streams by `local` frame. Returns first diverge.
pub fn first_desync(a: &[NdjsonSync], b: &[NdjsonSync]) -> Option<DesyncDiverge> {
    use std::collections::BTreeMap;
    let map_b: BTreeMap<u32, &NdjsonSync> = b.iter().map(|s| (s.local, s)).collect();
    for sa in a {
        let Some(sb) = map_b.get(&sa.local) else {
            continue;
        };
        let mut reasons = Vec::new();
        if sa.checksum != sb.checksum {
            reasons.push("checksum");
        }
        if sa.sync_percent != sb.sync_percent {
            reasons.push("sync_percent");
        }
        if sa.sync_stocks != sb.sync_stocks {
            reasons.push("sync_stocks");
        }
        if sa.sync_loc_x != sb.sync_loc_x {
            reasons.push("sync_locX");
        }
        if sa.sync_loc_y != sb.sync_loc_y {
            reasons.push("sync_locY");
        }
        if sa.sync_anim != sb.sync_anim {
            reasons.push("sync_anim");
        }
        if sa.sync_facing != sb.sync_facing {
            reasons.push("sync_facing");
        }
        if !reasons.is_empty() {
            return Some(DesyncDiverge {
                local: sa.local,
                reasons,
            });
        }
    }
    None
}

fn json_u32(line: &str, key: &str) -> Result<u32, String> {
    let v = json_raw(line, key)?;
    v.parse::<u32>()
        .map_err(|_| format!("bad u32 for {key}: {v}"))
}

fn json_i64(line: &str, key: &str) -> Result<i64, String> {
    let v = json_raw(line, key)?;
    v.parse::<i64>()
        .map_err(|_| format!("bad i64 for {key}: {v}"))
}

fn json_bool(line: &str, key: &str) -> Result<bool, String> {
    match json_raw(line, key)?.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("bad bool for {key}: {other}")),
    }
}

fn json_opt_u32(line: &str, key: &str) -> Result<Option<u32>, String> {
    let v = json_raw(line, key)?;
    if v == "null" {
        return Ok(None);
    }
    Ok(Some(
        v.parse::<u32>()
            .map_err(|_| format!("bad opt u32 for {key}: {v}"))?,
    ))
}

fn json_opt_f32(line: &str, key: &str) -> Result<Option<f32>, String> {
    let v = json_raw(line, key)?;
    if v == "null" {
        return Ok(None);
    }
    Ok(Some(
        v.parse::<f32>()
            .map_err(|_| format!("bad opt f32 for {key}: {v}"))?,
    ))
}

fn json_opt_u8(line: &str, key: &str) -> Result<Option<u8>, String> {
    let v = json_raw(line, key)?;
    if v == "null" {
        return Ok(None);
    }
    Ok(Some(
        v.parse::<u8>()
            .map_err(|_| format!("bad opt u8 for {key}: {v}"))?,
    ))
}

fn json_opt_i8(line: &str, key: &str) -> Result<Option<i8>, String> {
    let v = json_raw(line, key)?;
    if v == "null" {
        return Ok(None);
    }
    Ok(Some(
        v.parse::<i8>()
            .map_err(|_| format!("bad opt i8 for {key}: {v}"))?,
    ))
}

fn json_raw(line: &str, key: &str) -> Result<String, String> {
    let patterns = [format!("\"{key}\":"), format!("\"{key}\": ")];
    let mut start = None;
    for p in &patterns {
        if let Some(i) = line.find(p) {
            start = Some(i + p.len());
            break;
        }
    }
    let start = start.ok_or_else(|| format!("missing key {key}"))?;
    let rest = line[start..].trim_start();
    if rest.starts_with('"') {
        let end = rest[1..]
            .find('"')
            .ok_or_else(|| format!("unclosed string for {key}"))?
            + 1;
        return Ok(rest[1..end].to_string());
    }
    let end = rest
        .find(|c: char| c == ',' || c == '}')
        .unwrap_or(rest.len());
    Ok(rest[..end].trim().to_string())
}

#[derive(Debug)]
pub struct Session {
    pub local_player: u8,
    pub latest_confirmed: u32,
    pub is_predicting: bool,
    pub predicted: Option<PlayerFrame>,
    pub remote: Vec<PlayerFrame>,
    pub frames_to_advance: u32,
    pub start_rollback: Option<u32>,
    pub stop_rollback: Option<u32>,
    pub log: Vec<FrameLogEntry>,
}

impl Session {
    pub fn new(local_player: u8) -> Self {
        Self {
            local_player,
            latest_confirmed: 0,
            is_predicting: false,
            predicted: None,
            remote: Vec::new(),
            frames_to_advance: 1,
            start_rollback: None,
            stop_rollback: None,
            log: Vec::new(),
        }
    }

    pub fn push_remote(&mut self, f: PlayerFrame) {
        self.remote
            .retain(|x| x.frame != f.frame || x.player_idx != f.player_idx);
        self.remote.push(f);
        self.remote.sort_by_key(|x| x.frame);
    }

    fn latest_remote(&self, player_idx: u8) -> u32 {
        self.remote
            .iter()
            .filter(|f| f.player_idx == player_idx)
            .map(|f| f.frame)
            .max()
            .unwrap_or(0)
    }

    fn find_remote(&self, frame: u32, player_idx: u8) -> Option<&PlayerFrame> {
        self.remote
            .iter()
            .find(|f| f.frame == frame && f.player_idx == player_idx)
    }

    pub fn should_rollback(&self, loc_frame: u32, player_idx: u8) -> bool {
        loc_frame > self.latest_confirmed && self.latest_remote(player_idx) > self.latest_confirmed
    }

    pub fn in_rollback_mode(&self, loc_frame: u32, player_idx: u8) -> bool {
        let count = self
            .remote
            .iter()
            .filter(|f| f.player_idx == player_idx)
            .count() as u32;
        loc_frame > GAME_FULL_START_FRAME && count >= MAX_ROLLBACK_FRAMES
    }

    pub fn remote_inputs_for(&mut self, loc_frame: u32, player_idx: u8) -> PlayerFrame {
        if !self.in_rollback_mode(loc_frame, player_idx) {
            self.is_predicting = false;
            return self
                .find_remote(loc_frame, player_idx)
                .cloned()
                .unwrap_or(PlayerFrame {
                    frame: loc_frame,
                    player_idx,
                    pad: Pad::blank(),
                });
        }
        if let Some(r) = self.find_remote(loc_frame, player_idx).cloned() {
            self.is_predicting = false;
            return r;
        }
        let pred_frame = self.latest_confirmed;
        if let Some(prev) = self.find_remote(pred_frame, player_idx).cloned() {
            let mut predicted = prev.clone();
            predicted.frame = loc_frame;
            self.predicted = Some(prev);
            self.is_predicting = true;
            predicted
        } else {
            self.is_predicting = false;
            PlayerFrame {
                frame: loc_frame,
                player_idx,
                pad: Pad::blank(),
            }
        }
    }

    pub fn update_sync(&mut self, loc_frame: &mut u32, player_idx: u8, sync_checksum: u32) {
        let remote_frame = self.latest_remote(player_idx);
        let final_frame = remote_frame.min(*loc_frame);
        let mut synchronized = true;

        if self.is_predicting
            && self.should_rollback(*loc_frame, player_idx)
            && self.latest_confirmed > 0
        {
            if let Some(pred) = self.predicted.clone() {
                for i in (self.latest_confirmed + 1)..=final_frame {
                    match self.find_remote(i, player_idx) {
                        Some(remote) if remote.pad != pred.pad => {
                            self.latest_confirmed = i - 1;
                            synchronized = false;
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        self.start_rollback = None;
        self.stop_rollback = None;

        if synchronized {
            self.latest_confirmed = final_frame;
            self.frames_to_advance = 1;
        } else {
            self.frames_to_advance = *loc_frame - self.latest_confirmed + 1;
            self.stop_rollback = Some(*loc_frame);
            *loc_frame = self.latest_confirmed;
            self.start_rollback = Some(*loc_frame);
        }

        self.log.push(FrameLogEntry {
            local_frame: *loc_frame,
            latest_confirmed: self.latest_confirmed,
            predicting: self.is_predicting,
            rollback_start: self.start_rollback,
            rollback_stop: self.stop_rollback,
            frames_to_advance: self.frames_to_advance,
            sync_checksum,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn pad(buttons: u16) -> Pad {
        Pad {
            buttons,
            stick_x: 0,
            stick_y: 0,
        }
    }

    #[test]
    fn should_rollback_requires_frames_after_sync() {
        let mut s = Session::new(0);
        s.latest_confirmed = 160;
        s.push_remote(PlayerFrame {
            frame: 160,
            player_idx: 1,
            pad: pad(0),
        });
        assert!(!s.should_rollback(160, 1));
        s.push_remote(PlayerFrame {
            frame: 161,
            player_idx: 1,
            pad: pad(0),
        });
        assert!(s.should_rollback(162, 1));
    }

    #[test]
    fn predict_miss_sets_rollback_range_and_resim_count() {
        let mut s = Session::new(0);
        for f in 146..=155 {
            s.push_remote(PlayerFrame {
                frame: f,
                player_idx: 1,
                pad: pad(0),
            });
        }
        s.latest_confirmed = 155;
        let mut loc = 158;
        let _ = s.remote_inputs_for(loc, 1);
        assert!(s.is_predicting);
        s.push_remote(PlayerFrame {
            frame: 156,
            player_idx: 1,
            pad: pad(1),
        });
        s.push_remote(PlayerFrame {
            frame: 157,
            player_idx: 1,
            pad: pad(1),
        });
        s.push_remote(PlayerFrame {
            frame: 158,
            player_idx: 1,
            pad: pad(1),
        });
        s.update_sync(&mut loc, 1, 0xDEAD);
        let last = s.log.last().unwrap();
        assert_eq!(last.rollback_start, Some(155));
        assert_eq!(last.rollback_stop, Some(158));
        assert_eq!(last.frames_to_advance, 4);
        assert_eq!(last.sync_checksum, 0xDEAD);
        assert_eq!(loc, 155);
    }

    #[test]
    fn matching_prediction_advances_confirmed_without_rollback() {
        let mut s = Session::new(0);
        for f in 146..=155 {
            s.push_remote(PlayerFrame {
                frame: f,
                player_idx: 1,
                pad: pad(7),
            });
        }
        s.latest_confirmed = 155;
        let mut loc = 156;
        let _ = s.remote_inputs_for(loc, 1);
        s.push_remote(PlayerFrame {
            frame: 156,
            player_idx: 1,
            pad: pad(7),
        });
        s.update_sync(&mut loc, 1, 42);
        let last = s.log.last().unwrap();
        assert!(last.rollback_start.is_none());
        assert_eq!(last.latest_confirmed, 156);
        assert_eq!(last.frames_to_advance, 1);
    }

    #[test]
    fn fixture_ndjson_predict_miss_matches_join_keys() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/predict_miss.ndjson");
        let raw = fs::read_to_string(path).expect("fixture");
        let syncs = parse_sync_events(&raw).expect("parse");
        assert_eq!(syncs.len(), 2);
        assert!(syncs[0].rb_start.is_none());
        let miss = &syncs[1];
        assert_eq!(miss.local, 155);
        assert_eq!(miss.confirmed, 155);
        assert_eq!(miss.rb_start, Some(155));
        assert_eq!(miss.rb_stop, Some(158));
        assert_eq!(miss.frames_to_advance, Some(4));
        assert_eq!(miss.checksum, 291);
        assert!(miss.predicting);
    }

    #[test]
    fn dual_fixture_reports_first_checksum_diverge() {
        let a = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/peer_a.ndjson"
        ))
        .expect("peer_a");
        let b = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/peer_b.ndjson"
        ))
        .expect("peer_b");
        let sa = parse_sync_events(&a).expect("parse a");
        let sb = parse_sync_events(&b).expect("parse b");
        let d = first_desync(&sa, &sb).expect("should diverge");
        assert_eq!(d.local, 161);
        assert!(d.reasons.contains(&"checksum"));
        assert!(d.reasons.contains(&"sync_stocks"));
    }

    #[test]
    fn dual_fixture_identical_logs_no_desync() {
        let a = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/peer_a.ndjson"
        ))
        .expect("peer_a");
        let sa = parse_sync_events(&a).expect("parse a");
        assert!(first_desync(&sa, &sa).is_none());
    }
}
