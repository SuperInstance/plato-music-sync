//! Groove tracking — measuring how "in the groove" the fleet is.
//!
//! Groove = alignment of ticks across rooms. Each room's tick should land on the
//! expected phase within the master cycle. The groove score ranges from 0.0 (chaos)
//! to 1.0 (perfect sync). When groove drops below a threshold, sync correction is needed.

/// A tick event from a room.
#[derive(Debug, Clone)]
pub struct TickEvent {
    pub room_name: String,
    pub expected_phase: f64,
    pub actual_phase: f64,
    pub timestamp: f64,
}

/// The groove tracker measures fleet alignment over time.
#[derive(Debug, Clone)]
pub struct GrooveTracker {
    /// Recent tick events (ring buffer).
    events: Vec<TickEvent>,
    /// Maximum events to track.
    window_size: usize,
    /// Groove threshold below which correction is triggered.
    threshold: f64,
    /// Current groove score.
    current_groove: f64,
}

impl GrooveTracker {
    pub fn new(window_size: usize, threshold: f64) -> Self {
        Self {
            events: Vec::with_capacity(window_size),
            window_size,
            threshold,
            current_groove: 1.0,
        }
    }

    /// Record a tick event and update the groove score.
    pub fn record_tick(&mut self, event: TickEvent) {
        if self.events.len() >= self.window_size {
            self.events.remove(0);
        }
        self.events.push(event);
        self.recompute_groove();
    }

    /// Record a perfect tick (no phase error).
    pub fn record_perfect_tick(&mut self, room_name: &str, phase: f64, timestamp: f64) {
        self.record_tick(TickEvent {
            room_name: room_name.to_string(),
            expected_phase: phase,
            actual_phase: phase,
            timestamp,
        });
    }

    /// Recompute the groove score from recent events.
    fn recompute_groove(&mut self) {
        if self.events.is_empty() {
            self.current_groove = 1.0;
            return;
        }
        let total_error: f64 = self.events.iter().map(|e| {
            let err = (e.expected_phase - e.actual_phase).abs();
            // Phase wrapping: minimum of err and 1-err
            err.min(1.0 - err)
        }).sum();
        let avg_error = total_error / self.events.len() as f64;
        // Groove: 1.0 when error is 0, approaches 0.0 as error increases
        // Using exponential decay: groove = exp(-error * 20)
        self.current_groove = (-avg_error * 20.0).exp().clamp(0.0, 1.0);
    }

    /// Get the current groove score.
    pub fn groove(&self) -> f64 {
        self.current_groove
    }

    /// Check if groove is below threshold (sync correction needed).
    pub fn needs_correction(&self) -> bool {
        self.current_groove < self.threshold
    }

    /// Get the threshold.
    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    /// Number of tracked events.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Reset the tracker.
    pub fn reset(&mut self) {
        self.events.clear();
        self.current_groove = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_sync() {
        let mut tracker = GrooveTracker::new(100, 0.8);
        for i in 0..50 {
            let phase = i as f64 / 50.0;
            tracker.record_perfect_tick("engine", phase, i as f64);
        }
        assert!(tracker.groove() > 0.99);
    }

    #[test]
    fn test_one_room_late() {
        let mut tracker = GrooveTracker::new(100, 0.8);
        // Perfect ticks for most rooms
        for i in 0..40 {
            let phase = i as f64 / 50.0;
            tracker.record_perfect_tick("engine", phase, i as f64);
            tracker.record_perfect_tick("backdeck", phase, i as f64);
        }
        // One room ticks late
        for i in 40..50 {
            let phase = i as f64 / 50.0;
            tracker.record_perfect_tick("engine", phase, i as f64);
            tracker.record_tick(TickEvent {
                room_name: "backdeck".into(),
                expected_phase: phase,
                actual_phase: phase + 0.3, // 30% late
                timestamp: i as f64,
            });
        }
        assert!(tracker.groove() < 0.9);
    }

    #[test]
    fn test_groove_recovers() {
        let mut tracker = GrooveTracker::new(20, 0.8);
        // Add some bad ticks
        for i in 0..10 {
            tracker.record_tick(TickEvent {
                room_name: "engine".into(),
                expected_phase: i as f64 / 10.0,
                actual_phase: i as f64 / 10.0 + 0.2,
                timestamp: i as f64,
            });
        }
        assert!(tracker.groove() < 0.9);
        // Now recover with perfect ticks
        for i in 0..20 {
            let phase = i as f64 / 20.0;
            tracker.record_perfect_tick("engine", phase, i as f64 + 10.0);
        }
        assert!(tracker.groove() > 0.9);
    }

    #[test]
    fn test_threshold_detection() {
        let mut tracker = GrooveTracker::new(50, 0.8);
        // Perfect ticks — no correction needed
        for i in 0..30 {
            tracker.record_perfect_tick("engine", i as f64 / 30.0, i as f64);
        }
        assert!(!tracker.needs_correction());
        // Add bad ticks
        for i in 0..30 {
            tracker.record_tick(TickEvent {
                room_name: "engine".into(),
                expected_phase: i as f64 / 30.0,
                actual_phase: i as f64 / 30.0 + 0.15,
                timestamp: i as f64 + 30.0,
            });
        }
        assert!(tracker.needs_correction());
    }
}
