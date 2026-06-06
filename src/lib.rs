//! # plato-music-sync
//!
//! Music cognition patterns for synchronizing Plato rooms.
//!
//! The key insight: rooms ticking at different frequencies form a polyrhythmic ensemble.
//! The engine room ticks at 0.2 Hz (slow bass), the backdeck at 2 Hz (fast percussion),
//! the galley at 0.017 Hz (ambient drone). Music cognition provides the tools to keep
//! them in sync.
//!
//! ## Modules
//!
//! - [`polyrhythm`] — Coordinate rooms with different tick rates using LCM-based scheduling
//! - [`groove`] — Measure fleet alignment with groove scores (0.0–1.0)
//! - [`counterpoint`] — Detect productive vs wasteful room interactions via motion analysis
//! - [`cadence`] — Detect resolution patterns (perfect, deceptive, half cadences)
//! - [`tempo`] — Adaptive tick rate adjustment with tempo curves

pub mod polyrhythm;
pub mod groove;
pub mod counterpoint;
pub mod cadence;
pub mod tempo;

pub use polyrhythm::PolyrhythmicScheduler;
pub use groove::GrooveTracker;
pub use counterpoint::CounterpointAnalyzer;
pub use cadence::CadenceDetector;
pub use tempo::TempoMap;
