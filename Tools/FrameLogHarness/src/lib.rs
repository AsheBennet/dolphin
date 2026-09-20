//! Standalone model of Brawlback's GGPO-shaped updateSync / shouldRollback /
//! getRemoteInputs predict path (AsheBennet/dolphin savestates-efficiency-v2).
//! Not a full emulator — proves decision + frame-log shape until we can PR in-tree.

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

    /// Mirror getRemoteInputs predict branch.
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

    /// Mirror updateSync: on predict-miss, rollback range + frames_to_advance.
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
}
