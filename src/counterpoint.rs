//! Counterpoint analysis — detect productive vs wasteful room interactions.
//!
//! Inspired by species counterpoint in music theory:
//! - **Contrary motion**: one room's value goes up while another goes down (productive)
//! - **Parallel motion**: both go same direction (potentially wasteful/redundant)
//! - **Oblique motion**: one stable, other changing (normal operation)
//!
//! The MotionScore quantifies the quality of room coordination.

/// Direction of a room's value change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Stable,
}

/// Type of motion between two rooms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionType {
    /// Both rooms move in opposite directions — productive interaction.
    Contrary,
    /// Both rooms move in the same direction — potentially redundant.
    Parallel,
    /// One room stable, other changing — normal operation.
    Oblique,
}

/// A snapshot of a room's value at a point in time.
#[derive(Debug, Clone)]
pub struct RoomSnapshot {
    pub name: String,
    pub value: f64,
}

/// Score quantifying the quality of room coordination.
#[derive(Debug, Clone)]
pub struct MotionScore {
    /// Fraction of contrary motion (productive).
    pub contrary_ratio: f64,
    /// Fraction of parallel motion (potentially wasteful).
    pub parallel_ratio: f64,
    /// Fraction of oblique motion (neutral).
    pub oblique_ratio: f64,
    /// Overall quality: 0.0 (bad) to 1.0 (excellent).
    pub quality: f64,
}

/// The counterpoint analyzer detects motion patterns between room pairs.
#[derive(Debug, Clone)]
pub struct CounterpointAnalyzer {
    /// History of snapshots per room.
    history: Vec<Vec<RoomSnapshot>>,
    /// Maximum history depth.
    max_history: usize,
}

impl CounterpointAnalyzer {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Vec::with_capacity(max_history),
            max_history,
        }
    }

    /// Record a snapshot of all rooms at a point in time.
    pub fn record(&mut self, snapshot: Vec<RoomSnapshot>) {
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(snapshot);
    }

    /// Determine the direction of change for a room between two snapshots.
    pub fn direction(snapshots: &[RoomSnapshot], room_name: &str) -> Option<Direction> {
        let values: Vec<f64> = snapshots.iter()
            .filter_map(|s| if s.name == room_name { Some(s.value) } else { None })
            .collect();
        if values.len() < 2 {
            return None;
        }
        let diff = values[values.len() - 1] - values[values.len() - 2];
        if diff > 1e-6 {
            Some(Direction::Up)
        } else if diff < -1e-6 {
            Some(Direction::Down)
        } else {
            Some(Direction::Stable)
        }
    }

    /// Classify the motion type between two rooms.
    pub fn classify_motion(&self, room_a: &str, room_b: &str) -> Option<MotionType> {
        if self.history.len() < 2 {
            return None;
        }
        let latest = &self.history[self.history.len() - 1];
        let prev = &self.history[self.history.len() - 2];
        
        let dir_a = Self::direction_pair(prev, latest, room_a)?;
        let dir_b = Self::direction_pair(prev, latest, room_b)?;
        
        Some(match (dir_a, dir_b) {
            (Direction::Up, Direction::Down) | (Direction::Down, Direction::Up) => MotionType::Contrary,
            (Direction::Up, Direction::Up) | (Direction::Down, Direction::Down) => MotionType::Parallel,
            (Direction::Stable, _) | (_, Direction::Stable) => MotionType::Oblique,
        })
    }

    fn direction_pair(prev: &[RoomSnapshot], curr: &[RoomSnapshot], room: &str) -> Option<Direction> {
        let prev_val = prev.iter().find(|s| s.name == room)?.value;
        let curr_val = curr.iter().find(|s| s.name == room)?.value;
        let diff = curr_val - prev_val;
        if diff > 1e-6 {
            Some(Direction::Up)
        } else if diff < -1e-6 {
            Some(Direction::Down)
        } else {
            Some(Direction::Stable)
        }
    }

    /// Compute the MotionScore for a pair of rooms over the entire history.
    pub fn motion_score(&self, room_a: &str, room_b: &str) -> MotionScore {
        let mut contrary = 0usize;
        let mut parallel = 0usize;
        let mut oblique = 0usize;
        
        for i in 1..self.history.len() {
            let prev = &self.history[i - 1];
            let curr = &self.history[i];
            
            if let (Some(dir_a), Some(dir_b)) = (Self::direction_pair(prev, curr, room_a), Self::direction_pair(prev, curr, room_b)) {
                match (dir_a, dir_b) {
                    (Direction::Up, Direction::Down) | (Direction::Down, Direction::Up) => contrary += 1,
                    (Direction::Up, Direction::Up) | (Direction::Down, Direction::Down) => parallel += 1,
                    _ => oblique += 1,
                }
            }
        }
        
        let total = contrary + parallel + oblique;
        if total == 0 {
            return MotionScore {
                contrary_ratio: 0.0,
                parallel_ratio: 0.0,
                oblique_ratio: 1.0,
                quality: 0.5,
            };
        }
        
        let contrary_ratio = contrary as f64 / total as f64;
        let parallel_ratio = parallel as f64 / total as f64;
        let oblique_ratio = oblique as f64 / total as f64;
        
        // Quality: contrary is good, oblique is neutral, parallel is bad
        let quality = (contrary_ratio * 1.0 + oblique_ratio * 0.5 + parallel_ratio * 0.2).clamp(0.0, 1.0);
        
        MotionScore {
            contrary_ratio,
            parallel_ratio,
            oblique_ratio,
            quality,
        }
    }

    /// Get the history length.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contrary_motion() {
        let mut analyzer = CounterpointAnalyzer::new(100);
        // Room A goes up, Room B goes down
        analyzer.record(vec![
            RoomSnapshot { name: "engine".into(), value: 50.0 },
            RoomSnapshot { name: "bilge".into(), value: 30.0 },
        ]);
        analyzer.record(vec![
            RoomSnapshot { name: "engine".into(), value: 60.0 },
            RoomSnapshot { name: "bilge".into(), value: 20.0 },
        ]);
        assert_eq!(analyzer.classify_motion("engine", "bilge"), Some(MotionType::Contrary));
    }

    #[test]
    fn test_parallel_motion() {
        let mut analyzer = CounterpointAnalyzer::new(100);
        analyzer.record(vec![
            RoomSnapshot { name: "engine".into(), value: 50.0 },
            RoomSnapshot { name: "bilge".into(), value: 30.0 },
        ]);
        analyzer.record(vec![
            RoomSnapshot { name: "engine".into(), value: 60.0 },
            RoomSnapshot { name: "bilge".into(), value: 40.0 },
        ]);
        assert_eq!(analyzer.classify_motion("engine", "bilge"), Some(MotionType::Parallel));
    }

    #[test]
    fn test_oblique_motion() {
        let mut analyzer = CounterpointAnalyzer::new(100);
        analyzer.record(vec![
            RoomSnapshot { name: "engine".into(), value: 50.0 },
            RoomSnapshot { name: "bilge".into(), value: 30.0 },
        ]);
        analyzer.record(vec![
            RoomSnapshot { name: "engine".into(), value: 50.0 },
            RoomSnapshot { name: "bilge".into(), value: 40.0 },
        ]);
        assert_eq!(analyzer.classify_motion("engine", "bilge"), Some(MotionType::Oblique));
    }

    #[test]
    fn test_motion_score_fishing_boat() {
        let mut analyzer = CounterpointAnalyzer::new(100);
        // Simulate engine heating up while bilge level goes down (contrary — productive)
        for i in 0..50 {
            analyzer.record(vec![
                RoomSnapshot { name: "engine".into(), value: 50.0 + i as f64 },
                RoomSnapshot { name: "bilge".into(), value: 30.0 - i as f64 * 0.5 },
            ]);
        }
        let score = analyzer.motion_score("engine", "bilge");
        assert!(score.contrary_ratio > 0.9);
        assert!(score.quality > 0.7);
    }

    #[test]
    fn test_productive_interaction_high_score() {
        let mut analyzer = CounterpointAnalyzer::new(100);
        // Mix of contrary and oblique — productive
        for i in 0..30 {
            let engine_val = if i % 3 == 0 { 50.0 } else { 50.0 + (i as f64 * 0.5).sin() * 10.0 };
            let bilge_val = if i % 3 == 0 { 30.0 + i as f64 } else { 30.0 - (i as f64 * 0.3).cos() * 5.0 };
            analyzer.record(vec![
                RoomSnapshot { name: "engine".into(), value: engine_val },
                RoomSnapshot { name: "bilge".into(), value: bilge_val },
            ]);
        }
        let score = analyzer.motion_score("engine", "bilge");
        assert!(score.quality > 0.3);
    }
}
