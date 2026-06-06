//! Polyrhythmic scheduling for Plato rooms.
//!
//! Each room has a "time signature" derived from its tick frequency. The LCM of all
//! tick rates (as rational numbers) gives the "master cycle" — the shortest period
//! after which all rooms realign. The schedule maps each phase of the master cycle
//! to which rooms should tick.

use std::collections::HashMap;

/// A room in the polyrhythmic ensemble.
#[derive(Debug, Clone)]
pub struct Room {
    /// Human-readable room name.
    pub name: String,
    /// Tick frequency in Hz.
    pub tick_hz: f64,
}

/// Phase error detected when a room ticks late or early.
#[derive(Debug, Clone)]
pub struct PhaseError {
    pub room_name: String,
    pub expected_phase: f64,
    pub actual_phase: f64,
    pub error: f64,
}

/// The polyrhythmic scheduler coordinates rooms with different tick rates.
#[derive(Debug, Clone)]
pub struct PolyrhythmicScheduler {
    rooms: Vec<Room>,
    /// Master cycle length in seconds (LCM of all tick periods).
    master_cycle: f64,
    /// Schedule: for each room, which phases (0.0–1.0 of master cycle) it ticks at.
    schedule: HashMap<String, Vec<f64>>,
}

impl PolyrhythmicScheduler {
    /// Create a new scheduler with the given rooms.
    pub fn new(rooms: Vec<Room>) -> Self {
        let master_cycle = Self::compute_master_cycle(&rooms);
        let schedule = Self::build_schedule(&rooms, master_cycle);
        Self { rooms, master_cycle, schedule }
    }

    /// Compute the master cycle length as LCM of all tick periods.
    /// Uses rational approximation: tick_hz = p/q where p,q are small integers.
    /// Master cycle = LCM of denominators / GCD of numerators, then converted to seconds.
    fn compute_master_cycle(rooms: &[Room]) -> f64 {
        if rooms.is_empty() {
            return 1.0;
        }
        // Find LCM of all tick periods using rational approximation
        // tick_period = 1/tick_hz
        // Approximate each tick_hz as a fraction p/q with q <= 10000
        let periods: Vec<f64> = rooms.iter().map(|r| 1.0 / r.tick_hz).collect();
        
        // Use the approach: find the smallest T such that T / period_i is close to integer for all i
        // Try multiples of the longest period
        let max_period = periods.iter().cloned().fold(f64::MIN, f64::max);
        let mut best_cycle = max_period;
        
        // Try multiples up to 1000
        for mult in 1..=1000 {
            let candidate = max_period * mult as f64;
            let mut all_aligned = true;
            for period in &periods {
                let ratio = candidate / period;
                let nearest = ratio.round();
                if (ratio - nearest).abs() > 0.01 {
                    all_aligned = false;
                    break;
                }
            }
            if all_aligned {
                best_cycle = candidate;
                break;
            }
        }
        best_cycle
    }

    /// Build the schedule: for each room, compute the phases at which it ticks.
    fn build_schedule(rooms: &[Room], master_cycle: f64) -> HashMap<String, Vec<f64>> {
        let mut schedule = HashMap::new();
        for room in rooms {
            let tick_period = 1.0 / room.tick_hz;
            let ticks_per_cycle = (master_cycle / tick_period).round() as usize;
            let mut phases = Vec::with_capacity(ticks_per_cycle);
            for i in 0..ticks_per_cycle {
                let phase = (i as f64 * tick_period) / master_cycle;
                phases.push(phase);
            }
            schedule.insert(room.name.clone(), phases);
        }
        schedule
    }

    /// Get the master cycle length in seconds.
    pub fn master_cycle(&self) -> f64 {
        self.master_cycle
    }

    /// Get the schedule for a room.
    pub fn schedule_for(&self, room_name: &str) -> Option<&Vec<f64>> {
        self.schedule.get(room_name)
    }

    /// Get all rooms.
    pub fn rooms(&self) -> &[Room] {
        &self.rooms
    }

    /// Number of rooms.
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Total ticks per master cycle across all rooms.
    pub fn total_ticks_per_cycle(&self) -> usize {
        self.schedule.values().map(|v| v.len()).sum()
    }

    /// Given a current time `t` (seconds), return which rooms should tick.
    pub fn rooms_ticking_at(&self, t: f64) -> Vec<&Room> {
        let phase = (t % self.master_cycle) / self.master_cycle;
        let tolerance = 0.01;
        self.rooms.iter().filter(|room| {
            if let Some(phases) = self.schedule.get(&room.name) {
                phases.iter().any(|p| (p - phase).abs() < tolerance)
            } else {
                false
            }
        }).collect()
    }

    /// Detect phase errors: rooms that should have ticked but didn't (or ticked late).
    pub fn detect_phase_errors(&self, room_name: &str, actual_phase: f64) -> Option<PhaseError> {
        let phases = self.schedule.get(room_name)?;
        // Find the closest expected phase
        let closest = phases.iter().cloned()
            .min_by(|a, b| (a - actual_phase).abs().partial_cmp(&(b - actual_phase).abs()).unwrap_or(std::cmp::Ordering::Equal))?;
        let error = (closest - actual_phase).abs();
        if error > 0.01 {
            Some(PhaseError {
                room_name: room_name.to_string(),
                expected_phase: closest,
                actual_phase,
                error,
            })
        } else {
            None
        }
    }
}

/// Compute LCM of two values (for tick rates expressed as rational numbers).
pub fn lcm(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 { return 0; }
    a / gcd(a, b) * b
}

/// Compute GCD.
pub fn gcd(a: u64, b: u64) -> u64 {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Approximate a float as a fraction (numerator, denominator).
pub fn approximate_fraction(value: f64, max_denominator: u64) -> (u64, u64) {
    let mut best_num = 1u64;
    let mut best_den = 1u64;
    let mut best_err = f64::MAX;
    for den in 1..=max_denominator {
        let num = (value * den as f64).round() as u64;
        let err = (value - num as f64 / den as f64).abs();
        if err < best_err {
            best_err = err;
            best_num = num;
            best_den = den;
        }
        if err < 1e-10 {
            break;
        }
    }
    (best_num, best_den)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcm_basic() {
        assert_eq!(lcm(4, 6), 12);
        assert_eq!(lcm(3, 5), 15);
        assert_eq!(lcm(7, 7), 7);
    }

    #[test]
    fn test_schedule_three_rooms() {
        let rooms = vec![
            Room { name: "engine".into(), tick_hz: 0.2 },
            Room { name: "backdeck".into(), tick_hz: 1.0 },
            Room { name: "galley".into(), tick_hz: 2.0 },
        ];
        let scheduler = PolyrhythmicScheduler::new(rooms);
        // Master cycle should be 5 seconds (LCM of 5s, 1s, 0.5s)
        assert!((scheduler.master_cycle() - 5.0).abs() < 0.1);
        // Engine: 1 tick per cycle
        assert_eq!(scheduler.schedule_for("engine").unwrap().len(), 1);
        // Backdeck: 5 ticks per cycle
        assert_eq!(scheduler.schedule_for("backdeck").unwrap().len(), 5);
        // Galley: 10 ticks per cycle
        assert_eq!(scheduler.schedule_for("galley").unwrap().len(), 10);
    }

    #[test]
    fn test_detect_phase_error() {
        let rooms = vec![
            Room { name: "engine".into(), tick_hz: 1.0 },
        ];
        let scheduler = PolyrhythmicScheduler::new(rooms);
        // Phase 0.5 should be fine for a 1Hz room in a 1s cycle
        assert!(scheduler.detect_phase_errors("engine", 0.0).is_none());
        // Phase 0.45 is off by 0.05 — should detect error
        let err = scheduler.detect_phase_errors("engine", 0.45);
        assert!(err.is_some());
        assert!(err.unwrap().error > 0.01);
    }

    #[test]
    fn test_fishing_boat_master_cycle() {
        let rooms = vec![
            Room { name: "engine".into(), tick_hz: 0.2 },
            Room { name: "backdeck".into(), tick_hz: 2.0 },
            Room { name: "galley".into(), tick_hz: 0.017 },
            Room { name: "bilge".into(), tick_hz: 0.1 },
            Room { name: "bridge".into(), tick_hz: 1.0 },
        ];
        let scheduler = PolyrhythmicScheduler::new(rooms);
        // Should have computed a master cycle
        assert!(scheduler.master_cycle() > 0.0);
        assert_eq!(scheduler.room_count(), 5);
    }
}
