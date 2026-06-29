use crate::monitor::SystemSnapshot;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Controls polling frequency based on window visibility and system state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PollMode {
    /// Full speed - window visible, active monitoring
    Active,
    /// Reduced speed - window hidden but still running
    Background,
    /// Minimum speed - system idle/locked
    Idle,
}

/// Configuration for polling behavior
#[derive(Debug, Clone)]
pub struct PollConfig {
    pub active_interval_ms: u64,
    pub background_interval_ms: u64,
    pub idle_interval_ms: u64,
    pub mode: PollMode,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            active_interval_ms: 1000,
            background_interval_ms: 5000,
            idle_interval_ms: 15000,
            mode: PollMode::Active,
        }
    }
}

/// The monitoring engine that manages data collection
pub struct MonitorEngine {
    config: PollConfig,
    current_snapshot: Arc<Mutex<Option<SystemSnapshot>>>,
    last_poll: Instant,
    maybe_idle_detected: bool,
    last_active_time: Instant,
}

impl MonitorEngine {
    pub fn new(config: PollConfig) -> Self {
        Self {
            config,
            current_snapshot: Arc::new(Mutex::new(None)),
            last_poll: Instant::now(),
            maybe_idle_detected: false,
            last_active_time: Instant::now(),
        }
    }

    /// Get a shared reference to the latest snapshot
    pub fn snapshot(&self) -> Arc<Mutex<Option<SystemSnapshot>>> {
        self.current_snapshot.clone()
    }

    /// Set the poll mode (called by UI when window state changes)
    pub fn set_mode(&mut self, mode: PollMode) {
        self.config.mode = mode;
        if mode == PollMode::Active {
            self.last_active_time = Instant::now();
            self.maybe_idle_detected = false;
        }
    }

    /// Get the current polling interval based on mode
    pub fn interval(&self) -> Duration {
        match self.config.mode {
            PollMode::Active => Duration::from_millis(self.config.active_interval_ms),
            PollMode::Background => Duration::from_millis(self.config.background_interval_ms),
            PollMode::Idle => Duration::from_millis(self.config.idle_interval_ms),
        }
    }

    /// Check if it's time to poll again
    pub fn should_poll(&self) -> bool {
        self.last_poll.elapsed() >= self.interval()
    }

    /// Run one collection cycle. Returns the new snapshot.
    pub fn poll(&mut self) -> SystemSnapshot {
        let snapshot = SystemSnapshot::collect();
        if let Ok(mut current) = self.current_snapshot.lock() {
            *current = Some(snapshot.clone());
        }
        self.last_poll = Instant::now();
        self.last_active_time = Instant::now();

        if self.config.mode == PollMode::Background
            && self.last_active_time.elapsed() > Duration::from_secs(120)
        {
            self.config.mode = PollMode::Idle;
            self.maybe_idle_detected = true;
        }

        snapshot
    }

    /// Collect a snapshot without updating engine state (for history recording)
    pub fn poll_snapshot(&self) -> SystemSnapshot {
        SystemSnapshot::collect()
    }

    /// Reset idle detection (call when user interacts)
    pub fn mark_active(&mut self) {
        self.last_active_time = Instant::now();
        if self.maybe_idle_detected && self.config.mode == PollMode::Idle {
            self.config.mode = PollMode::Background;
            self.maybe_idle_detected = false;
        }
    }

    /// Update active polling interval
    pub fn set_active_interval(&mut self, ms: u64) {
        self.config.active_interval_ms = ms.clamp(200, 10000);
    }
}

/// Simple rate limiter for resource control
pub struct RateLimiter {
    last: Instant,
    interval: Duration,
}

impl RateLimiter {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            last: Instant::now(),
            interval: Duration::from_millis(interval_ms),
        }
    }

    pub fn try_acquire(&mut self) -> bool {
        if self.last.elapsed() >= self.interval {
            self.last = Instant::now();
            true
        } else {
            false
        }
    }
}
