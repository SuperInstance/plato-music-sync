//! Cadence detection — detect resolution patterns in room state.
//!
//! Inspired by musical cadences:
//! - **Perfect cadence**: alarm → action → resolved (V→I equivalent)
//! - **Deceptive cadence**: alarm → action → new alarm (V→vi equivalent)
//! - **Half cadence**: alarm → still pending (...→V equivalent)
//!
//! Tracking cadence patterns over time enables predictive maintenance.

/// Type of cadence detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CadenceType {
    /// Alarm triggered → action taken → alarm resolved.
    Perfect,
    /// Alarm triggered → action taken → new alarm appeared.
    Deceptive,
    /// Alarm triggered → still pending resolution.
    Half,
}

/// A state transition event in a room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomState {
    Normal,
    Alarm,
    Action,
    Resolved,
}

/// A recorded cadence event.
#[derive(Debug, Clone)]
pub struct CadenceEvent {
    pub room_name: String,
    pub state: RoomState,
    pub tick: u64,
}

/// A completed cadence pattern.
#[derive(Debug, Clone)]
pub struct CadencePattern {
    pub cadence_type: CadenceType,
    pub room_name: String,
    pub start_tick: u64,
    pub end_tick: Option<u64>,
    pub events: Vec<CadenceEvent>,
}

/// The cadence detector tracks resolution patterns.
#[derive(Debug, Clone)]
pub struct CadenceDetector {
    /// Current state per room.
    room_states: std::collections::HashMap<String, RoomState>,
    /// Active sequences per room (alarm → ... sequences).
    active_sequences: std::collections::HashMap<String, Vec<CadenceEvent>>,
    /// Completed cadence patterns.
    patterns: Vec<CadencePattern>,
    /// Current tick counter.
    current_tick: u64,
}

impl CadenceDetector {
    pub fn new() -> Self {
        Self {
            room_states: std::collections::HashMap::new(),
            active_sequences: std::collections::HashMap::new(),
            patterns: Vec::new(),
            current_tick: 0,
        }
    }

    /// Advance the tick counter.
    pub fn tick(&mut self) {
        self.current_tick += 1;
    }

    /// Get current tick.
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Record a state change for a room.
    pub fn record_state(&mut self, room_name: &str, state: RoomState) -> Option<CadenceType> {
        let prev = self.room_states.insert(room_name.to_string(), state);
        let event = CadenceEvent {
            room_name: room_name.to_string(),
            state,
            tick: self.current_tick,
        };
        
        match state {
            RoomState::Alarm => {
                // Start a new sequence
                self.active_sequences.insert(room_name.to_string(), vec![event]);
                None
            }
            RoomState::Action => {
                // Continue the sequence
                if let Some(seq) = self.active_sequences.get_mut(room_name) {
                    seq.push(event);
                }
                None
            }
            RoomState::Resolved => {
                // Complete the sequence — perfect cadence
                if let Some(mut seq) = self.active_sequences.remove(room_name) {
                    seq.push(event);
                    let start_tick = seq.first().map(|e| e.tick).unwrap_or(self.current_tick);
                    self.patterns.push(CadencePattern {
                        cadence_type: CadenceType::Perfect,
                        room_name: room_name.to_string(),
                        start_tick,
                        end_tick: Some(self.current_tick),
                        events: seq,
                    });
                    Some(CadenceType::Perfect)
                } else {
                    None
                }
            }
            RoomState::Normal => {
                // If we had an active alarm that went back to normal without resolution
                // or if an alarm appeared after action (deceptive), handle it
                if let Some(seq) = self.active_sequences.get(room_name) {
                    // Check if last state was Action — if so, going to Normal without
                    // explicit Resolved is ambiguous. If last was Alarm, it's a half cadence.
                    // We don't finalize here — half cadences are detected by pending()
                }
                None
            }
        }
    }

    /// Record an alarm followed by action that leads to a new alarm (deceptive cadence).
    pub fn record_deceptive_transition(&mut self, room_name: &str) -> CadenceType {
        let event = CadenceEvent {
            room_name: room_name.to_string(),
            state: RoomState::Alarm,
            tick: self.current_tick,
        };
        
        if let Some(mut seq) = self.active_sequences.remove(room_name) {
            let new_event = CadenceEvent {
                room_name: room_name.to_string(),
                state: RoomState::Alarm,
                tick: self.current_tick,
            };
            seq.push(event);
            let start_tick = seq.first().map(|e| e.tick).unwrap_or(self.current_tick);
            self.patterns.push(CadencePattern {
                cadence_type: CadenceType::Deceptive,
                room_name: room_name.to_string(),
                start_tick,
                end_tick: Some(self.current_tick),
                events: seq,
            });
            // Start new alarm sequence
            self.active_sequences.insert(room_name.to_string(), vec![new_event]);
        }
        CadenceType::Deceptive
    }

    /// Check for half cadences — rooms with pending alarms.
    pub fn detect_half_cadences(&self) -> Vec<(&str, u64)> {
        self.active_sequences.iter()
            .filter_map(|(name, seq)| {
                if seq.first().map(|e| e.state) == Some(RoomState::Alarm) {
                    Some((name.as_str(), seq.first().unwrap().tick))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Finalize a half cadence (alarm still pending).
    pub fn finalize_half_cadence(&mut self, room_name: &str) -> Option<CadenceType> {
        if let Some(seq) = self.active_sequences.remove(room_name) {
            let start_tick = seq.first().map(|e| e.tick).unwrap_or(self.current_tick);
            self.patterns.push(CadencePattern {
                cadence_type: CadenceType::Half,
                room_name: room_name.to_string(),
                start_tick,
                end_tick: None,
                events: seq,
            });
            Some(CadenceType::Half)
        } else {
            None
        }
    }

    /// Get all completed patterns.
    pub fn patterns(&self) -> &[CadencePattern] {
        &self.patterns
    }

    /// Count patterns by type.
    pub fn count_by_type(&self, cadence_type: CadenceType) -> usize {
        self.patterns.iter().filter(|p| p.cadence_type == cadence_type).count()
    }

    /// Get cadence statistics.
    pub fn stats(&self) -> CadenceStats {
        let perfect = self.count_by_type(CadenceType::Perfect);
        let deceptive = self.count_by_type(CadenceType::Deceptive);
        let half = self.count_by_type(CadenceType::Half);
        CadenceStats { perfect, deceptive, half, total: perfect + deceptive + half }
    }
}

/// Summary statistics of cadence patterns.
#[derive(Debug, Clone)]
pub struct CadenceStats {
    pub perfect: usize,
    pub deceptive: usize,
    pub half: usize,
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_cadence() {
        let mut detector = CadenceDetector::new();
        detector.tick(); // tick 1
        let result = detector.record_state("engine", RoomState::Alarm);
        assert!(result.is_none());
        detector.tick(); // tick 2
        let result = detector.record_state("engine", RoomState::Action);
        assert!(result.is_none());
        detector.tick(); // tick 3
        let result = detector.record_state("engine", RoomState::Resolved);
        assert_eq!(result, Some(CadenceType::Perfect));
        assert_eq!(detector.count_by_type(CadenceType::Perfect), 1);
    }

    #[test]
    fn test_deceptive_cadence() {
        let mut detector = CadenceDetector::new();
        detector.tick();
        detector.record_state("engine", RoomState::Alarm);
        detector.tick();
        detector.record_state("engine", RoomState::Action);
        detector.tick();
        let result = detector.record_deceptive_transition("engine");
        assert_eq!(result, CadenceType::Deceptive);
        assert_eq!(detector.count_by_type(CadenceType::Deceptive), 1);
    }

    #[test]
    fn test_half_cadence() {
        let mut detector = CadenceDetector::new();
        detector.tick();
        detector.record_state("engine", RoomState::Alarm);
        // No action, no resolution — alarm still pending
        let half = detector.detect_half_cadences();
        assert_eq!(half.len(), 1);
        assert_eq!(half[0].0, "engine");
        detector.tick();
        let result = detector.finalize_half_cadence("engine");
        assert_eq!(result, Some(CadenceType::Half));
        assert_eq!(detector.count_by_type(CadenceType::Half), 1);
    }

    #[test]
    fn test_track_pattern_over_100_ticks() {
        let mut detector = CadenceDetector::new();
        // Simulate 100 ticks with alternating cadences
        for i in 0..25 {
            detector.tick(); // alarm
            detector.record_state("engine", RoomState::Alarm);
            detector.tick(); // action
            detector.record_state("engine", RoomState::Action);
            if i % 5 == 4 {
                // Every 5th cycle: deceptive
                detector.tick();
                detector.record_deceptive_transition("engine");
                // Resolve the new alarm
                detector.tick();
                detector.record_state("engine", RoomState::Action);
                detector.tick();
                detector.record_state("engine", RoomState::Resolved);
            } else {
                detector.tick(); // resolved
                detector.record_state("engine", RoomState::Resolved);
            }
            detector.tick(); // normal
            detector.record_state("engine", RoomState::Normal);
        }
        let stats = detector.stats();
        assert!(stats.perfect >= 15); // Most should be perfect
        assert!(stats.deceptive >= 3); // Some deceptive
        assert!(stats.total >= 20);
    }
}
