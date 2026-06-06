//! Tempo map — adaptive tick rate adjustment.
//!
//! Rooms can speed up (allegro) during crisis and slow down (adagio) during stable
//! periods. Tempo changes propagate through the fleet. This module connects to
//! agent-rubato's tempo curve model for smooth accelerando/ritardando transitions.

/// A tempo marking for a room.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TempoMarking {
    /// Very slow — deep stability.
    Grave,
    /// Slow — stable period.
    Adagio,
    /// Walking pace — normal operation.
    Andante,
    /// Moderate — slightly elevated activity.
    Moderato,
    /// Fast — active response.
    Allegro,
    /// Very fast — crisis mode.
    Presto,
}

impl TempoMarking {
    /// Convert to a tick rate multiplier.
    pub fn multiplier(&self) -> f64 {
        match self {
            TempoMarking::Grave => 0.25,
            TempoMarking::Adagio => 0.5,
            TempoMarking::Andante => 1.0,
            TempoMarking::Moderato => 1.5,
            TempoMarking::Allegro => 2.0,
            TempoMarking::Presto => 4.0,
        }
    }

    /// Get marking from a multiplier value.
    pub fn from_multiplier(mult: f64) -> Self {
        if mult <= 0.35 {
            TempoMarking::Grave
        } else if mult <= 0.75 {
            TempoMarking::Adagio
        } else if mult <= 1.25 {
            TempoMarking::Andante
        } else if mult <= 1.75 {
            TempoMarking::Moderato
        } else if mult <= 3.0 {
            TempoMarking::Allegro
        } else {
            TempoMarking::Presto
        }
    }
}

/// A tempo curve point for smooth transitions.
#[derive(Debug, Clone)]
pub struct TempoCurvePoint {
    pub tick: u64,
    pub multiplier: f64,
}

/// The tempo map manages adaptive tick rates across rooms.
#[derive(Debug, Clone)]
pub struct TempoMap {
    /// Base tick rate per room.
    base_rates: std::collections::HashMap<String, f64>,
    /// Current tempo multiplier per room.
    current_multipliers: std::collections::HashMap<String, f64>,
    /// Tempo curve for smooth transitions.
    curve: Vec<TempoCurvePoint>,
    /// Global crisis level (0.0 = calm, 1.0 = full crisis).
    crisis_level: f64,
}

impl TempoMap {
    pub fn new() -> Self {
        Self {
            base_rates: std::collections::HashMap::new(),
            current_multipliers: std::collections::HashMap::new(),
            curve: Vec::new(),
            crisis_level: 0.0,
        }
    }

    /// Add a room with its base tick rate.
    pub fn add_room(&mut self, name: &str, base_hz: f64) {
        self.base_rates.insert(name.to_string(), base_hz);
        self.current_multipliers.insert(name.to_string(), 1.0);
    }

    /// Get the effective tick rate for a room.
    pub fn effective_rate(&self, name: &str) -> f64 {
        let base = self.base_rates.get(name).copied().unwrap_or(1.0);
        let mult = self.current_multipliers.get(name).copied().unwrap_or(1.0);
        base * mult
    }

    /// Get the current multiplier for a room.
    pub fn multiplier(&self, name: &str) -> f64 {
        self.current_multipliers.get(name).copied().unwrap_or(1.0)
    }

    /// Get the tempo marking for a room.
    pub fn marking(&self, name: &str) -> TempoMarking {
        TempoMarking::from_multiplier(self.multiplier(name))
    }

    /// Speed up a room (e.g., during crisis).
    pub fn speed_up(&mut self, name: &str, factor: f64) {
        if let Some(mult) = self.current_multipliers.get_mut(name) {
            *mult = (*mult * factor).clamp(0.25, 4.0);
        }
    }

    /// Slow down a room (e.g., during stable period).
    pub fn slow_down(&mut self, name: &str, factor: f64) {
        if let Some(mult) = self.current_multipliers.get_mut(name) {
            *mult = (*mult / factor).clamp(0.25, 4.0);
        }
    }

    /// Set the crisis level, which automatically adjusts all rooms.
    pub fn set_crisis_level(&mut self, level: f64) {
        self.crisis_level = level.clamp(0.0, 1.0);
        // Crisis mapping: at level 0 → Andante (1.0), at level 1 → Presto (4.0)
        let global_mult = 1.0 + self.crisis_level * 3.0;
        for mult in self.current_multipliers.values_mut() {
            *mult = global_mult;
        }
    }

    /// Get the crisis level.
    pub fn crisis_level(&self) -> f64 {
        self.crisis_level
    }

    /// Propagate a tempo change from one room to others.
    pub fn propagate(&mut self, source: &str, targets: &[&str], blend: f64) {
        let source_mult = self.current_multipliers.get(source).copied().unwrap_or(1.0);
        for target in targets {
            if let Some(mult) = self.current_multipliers.get_mut(*target) {
                *mult = *mult * (1.0 - blend) + source_mult * blend;
            }
        }
    }

    /// Build a rubato curve (smooth tempo transition) over N ticks.
    pub fn build_rubato_curve(&mut self, from_mult: f64, to_mult: f64, ticks: u64) {
        self.curve.clear();
        for i in 0..=ticks {
            let t = i as f64 / ticks as f64;
            // Smooth S-curve interpolation (smoothstep)
            let t_smooth = t * t * (3.0 - 2.0 * t);
            let mult = from_mult + (to_mult - from_mult) * t_smooth;
            self.curve.push(TempoCurvePoint { tick: i, multiplier: mult });
        }
    }

    /// Get the curve multiplier at a specific tick.
    pub fn curve_multiplier_at(&self, tick: u64) -> f64 {
        if self.curve.is_empty() {
            return 1.0;
        }
        // Find the two surrounding points and interpolate
        let idx = self.curve.iter().position(|p| p.tick >= tick).unwrap_or(self.curve.len() - 1);
        if idx == 0 {
            return self.curve[0].multiplier;
        }
        let prev = &self.curve[idx - 1];
        let curr = &self.curve[idx];
        let t = (tick - prev.tick) as f64 / (curr.tick - prev.tick).max(1) as f64;
        prev.multiplier + (curr.multiplier - prev.multiplier) * t
    }

    /// Apply the rubato curve to a room.
    pub fn apply_curve_to(&mut self, name: &str, curve_offset: u64) {
        let value = self.curve_multiplier_at(curve_offset);
        if let Some(mult) = self.current_multipliers.get_mut(name) {
            *mult = value;
        }
    }

    /// Number of rooms tracked.
    pub fn room_count(&self) -> usize {
        self.base_rates.len()
    }

    /// Reset all rooms to Andante (1.0).
    pub fn reset(&mut self) {
        for mult in self.current_multipliers.values_mut() {
            *mult = 1.0;
        }
        self.crisis_level = 0.0;
        self.curve.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_up_during_crisis() {
        let mut map = TempoMap::new();
        map.add_room("engine", 1.0);
        assert_eq!(map.multiplier("engine"), 1.0);
        map.speed_up("engine", 2.0);
        assert!((map.multiplier("engine") - 2.0).abs() < 0.01);
        assert_eq!(map.marking("engine"), TempoMarking::Allegro);
    }

    #[test]
    fn test_slow_down_stable() {
        let mut map = TempoMap::new();
        map.add_room("engine", 1.0);
        map.speed_up("engine", 2.0);
        map.slow_down("engine", 2.0);
        assert!((map.multiplier("engine") - 1.0).abs() < 0.01);
        assert_eq!(map.marking("engine"), TempoMarking::Andante);
    }

    #[test]
    fn test_propagate_to_three_rooms() {
        let mut map = TempoMap::new();
        map.add_room("engine", 1.0);
        map.add_room("bilge", 0.5);
        map.add_room("bridge", 2.0);
        map.speed_up("engine", 3.0);
        // engine is now at 3.0
        map.propagate("engine", &["bilge", "bridge"], 0.5);
        let bilge_mult = map.multiplier("bilge");
        let bridge_mult = map.multiplier("bridge");
        // Blend 50%: new = old * 0.5 + 3.0 * 0.5
        assert!((bilge_mult - 2.0).abs() < 0.01); // 1.0*0.5 + 3.0*0.5 = 2.0
        assert!((bridge_mult - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_rubato_curve() {
        let mut map = TempoMap::new();
        map.build_rubato_curve(1.0, 2.0, 100);
        // At start: ~1.0
        assert!((map.curve_multiplier_at(0) - 1.0).abs() < 0.01);
        // At end: ~2.0
        assert!((map.curve_multiplier_at(100) - 2.0).abs() < 0.01);
        // In middle: smooth transition
        let mid = map.curve_multiplier_at(50);
        assert!(mid > 1.3 && mid < 1.7);
    }

    #[test]
    fn test_crisis_level() {
        let mut map = TempoMap::new();
        map.add_room("engine", 1.0);
        map.add_room("bilge", 0.5);
        map.set_crisis_level(1.0);
        assert!((map.multiplier("engine") - 4.0).abs() < 0.01);
        assert_eq!(map.marking("engine"), TempoMarking::Presto);
    }
}
