//! Integration tests for plato-music-sync

use plato_music_sync::*;

#[test]
fn test_full_fishing_boat_sync() {
    let rooms = vec![
        polyrhythm::Room { name: "engine".into(), tick_hz: 0.2 },
        polyrhythm::Room { name: "backdeck".into(), tick_hz: 2.0 },
        polyrhythm::Room { name: "galley".into(), tick_hz: 0.017 },
        polyrhythm::Room { name: "bilge".into(), tick_hz: 0.1 },
        polyrhythm::Room { name: "bridge".into(), tick_hz: 1.0 },
    ];
    let scheduler = PolyrhythmicScheduler::new(rooms);
    let mut groove = GrooveTracker::new(200, 0.8);
    let mut tempo = TempoMap::new();
    
    tempo.add_room("engine", 0.2);
    tempo.add_room("backdeck", 2.0);
    tempo.add_room("galley", 0.017);
    tempo.add_room("bilge", 0.1);
    tempo.add_room("bridge", 1.0);
    
    assert_eq!(scheduler.room_count(), 5);
    assert_eq!(tempo.room_count(), 5);
    
    // Simulate 100 seconds with perfect ticks
    let master = scheduler.master_cycle();
    for t in 0..100 {
        let t_sec = t as f64;
        let phase = (t_sec % master) / master;
        for room in scheduler.rooms() {
            if let Some(phases) = scheduler.schedule_for(&room.name) {
                if phases.iter().any(|p| (p - phase).abs() < 0.01) {
                    groove.record_perfect_tick(&room.name, phase, t_sec);
                }
            }
        }
    }
    assert!(groove.groove() > 0.9);
}

#[test]
fn test_engine_overheats_tempo_groove_counterpoint() {
    let mut groove = GrooveTracker::new(100, 0.7);
    let mut tempo = TempoMap::new();
    let mut counterpoint = CounterpointAnalyzer::new(100);
    let mut cadence = CadenceDetector::new();
    
    tempo.add_room("engine", 1.0);
    tempo.add_room("bilge", 0.5);
    
    // Phase 1: Normal operation (ticks 0-30)
    for t in 0..30u64 {
        cadence.tick();
        groove.record_perfect_tick("engine", (t as f64 % 10.0) / 10.0, t as f64);
        groove.record_perfect_tick("bilge", (t as f64 % 5.0) / 5.0, t as f64);
        counterpoint.record(vec![
            counterpoint::RoomSnapshot { name: "engine".into(), value: 50.0 },
            counterpoint::RoomSnapshot { name: "bilge".into(), value: 30.0 },
        ]);
    }
    assert!(groove.groove() > 0.9);
    
    // Phase 2: Engine overheats (ticks 30-50)
    cadence.record_state("engine", cadence::RoomState::Alarm);
    tempo.speed_up("engine", 3.0);
    for t in 30..50 {
        cadence.tick();
        let phase = (t as f64 % 10.0) / 10.0;
        groove.record_tick(groove::TickEvent {
            room_name: "engine".into(),
            expected_phase: phase,
            actual_phase: phase + 0.1, // Engine is off-beat
            timestamp: t as f64,
        });
        counterpoint.record(vec![
            counterpoint::RoomSnapshot { name: "engine".into(), value: 50.0 + (t - 30) as f64 * 2.0 },
            counterpoint::RoomSnapshot { name: "bilge".into(), value: 30.0 - (t - 30) as f64 },
        ]);
    }
    // Groove should have dropped
    assert!(groove.groove() < 0.95);
    
    // Phase 3: Action and resolution
    cadence.tick();
    cadence.record_state("engine", cadence::RoomState::Action);
    cadence.tick();
    let result = cadence.record_state("engine", cadence::RoomState::Resolved);
    assert_eq!(result, Some(cadence::CadenceType::Perfect));
    
    // Counterpoint should show contrary motion (engine up, bilge down)
    let score = counterpoint.motion_score("engine", "bilge");
    assert!(score.contrary_ratio > 0.3, "contrary_ratio was {}", score.contrary_ratio);
    
    // Engine tempo should be elevated
    assert!(tempo.effective_rate("engine") > 1.0);
}

#[test]
fn test_four_rooms_groove_one_goes_offline() {
    let mut groove = GrooveTracker::new(200, 0.8);
    let mut tempo = TempoMap::new();
    
    for name in &["engine", "bilge", "bridge", "galley"] {
        tempo.add_room(name, 1.0);
    }
    
    // All rooms in perfect sync for 50 ticks
    for t in 0..50 {
        let phase = t as f64 / 50.0;
        for name in &["engine", "bilge", "bridge", "galley"] {
            groove.record_perfect_tick(name, phase, t as f64);
        }
    }
    assert!(groove.groove() > 0.99);
    
    // Galley goes offline — no ticks for 30 ticks
    for t in 50..80 {
        let phase = t as f64 / 50.0;
        for name in &["engine", "bilge", "bridge"] {
            groove.record_perfect_tick(name, phase, t as f64);
        }
        // Galley: record late/wrong ticks
        groove.record_tick(groove::TickEvent {
            room_name: "galley".into(),
            expected_phase: phase,
            actual_phase: phase + 0.25,
            timestamp: t as f64,
        });
    }
    // Groove should have dropped
    assert!(groove.groove() < 0.9);
    assert!(groove.needs_correction());
}

#[test]
fn test_cadence_pattern_over_500_ticks() {
    let mut cadence = CadenceDetector::new();
    let mut tempo = TempoMap::new();
    tempo.add_room("engine", 1.0);
    
    let mut perfect_count = 0usize;
    let mut deceptive_count = 0usize;
    
    for cycle in 0..100 {
        // Alarm
        cadence.tick();
        cadence.record_state("engine", cadence::RoomState::Alarm);
        tempo.speed_up("engine", 1.1);
        
        // Action
        cadence.tick();
        cadence.record_state("engine", cadence::RoomState::Action);
        
        // Resolution
        cadence.tick();
        if cycle % 7 == 6 {
            // Every 7th cycle: deceptive (new alarm instead of resolution)
            cadence.record_deceptive_transition("engine");
            deceptive_count += 1;
            // Resolve the second alarm
            cadence.tick();
            cadence.record_state("engine", cadence::RoomState::Action);
            cadence.tick();
            cadence.record_state("engine", cadence::RoomState::Resolved);
            perfect_count += 1;
        } else {
            cadence.record_state("engine", cadence::RoomState::Resolved);
            perfect_count += 1;
        }
        
        // Return to normal
        cadence.tick();
        cadence.record_state("engine", cadence::RoomState::Normal);
        tempo.reset();
    }
    
    let stats = cadence.stats();
    assert!(stats.perfect >= 90);
    assert!(stats.deceptive >= 10);
    assert!(stats.total >= 100);
    assert!(cadence.current_tick() > 400, "tick was {}", cadence.current_tick());
}
