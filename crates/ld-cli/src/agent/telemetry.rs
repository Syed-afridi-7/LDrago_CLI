use ld_policy::SafetyVerdict;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

static TOTAL_TOKENS: AtomicU32 = AtomicU32::new(0);
static TOTAL_DURATION_MS: AtomicU64 = AtomicU64::new(0);
static SAFETY_EVENTS: Mutex<Vec<SafetyEventRecord>> = Mutex::new(Vec::new());

/// In-memory representation of a safety audit record.
#[derive(Debug, Clone)]
pub struct SafetyEventRecord {
    pub verdict: SafetyVerdict,
    pub command: String,
    pub timestamp: std::time::Instant,
}

/// Consolidated snapshot of inference telemetry metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelemetryStats {
    pub total_tokens: u32,
    pub total_duration_ms: u64,
}

/// [TELEMETRY HOOK] Record inference telemetry for agent reasoning steps.
///
/// This hook silently tracks token throughput and model generation latency in the background.
/// Deferred UI rendering: Raw telemetry events are NOT written directly to standard output to preserve
/// clean streaming terminal UX and avoid disrupting interactive CLI dialogs. Instead, metrics are
/// accumulated in background atomic registers for subsequent polling by TUI status bars or post-run
/// execution summaries.
pub fn record_inference_telemetry(tokens: u32, duration: Duration) {
    TOTAL_TOKENS.fetch_add(tokens, Ordering::Relaxed);
    TOTAL_DURATION_MS.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
    // [TELEMETRY HOOK] Deferred rendering register updated without stdout emission.
}

/// [SAFETY AUDIT HOOK] Log a security gate policy evaluation event.
///
/// This hook captures command verdicts (Safe vs Violation) triggered during agent tool execution.
/// Deferred UI rendering: Safety and audit events are captured silently in an in-memory audit log
/// without stdout clutter, allowing zero-trust auditing and post-execution security inspection while
/// maintaining pristine terminal display.
pub fn log_safety_event(verdict: &SafetyVerdict, command: &str) {
    if let Ok(mut events) = SAFETY_EVENTS.lock() {
        events.push(SafetyEventRecord {
            verdict: verdict.clone(),
            command: command.to_string(),
            timestamp: std::time::Instant::now(),
        });
    }
    // [SAFETY AUDIT HOOK] Event registered in background audit store.
}

/// Retrieve the current inference telemetry totals.
pub fn get_telemetry_stats() -> TelemetryStats {
    TelemetryStats {
        total_tokens: TOTAL_TOKENS.load(Ordering::Relaxed),
        total_duration_ms: TOTAL_DURATION_MS.load(Ordering::Relaxed),
    }
}

/// Retrieve a snapshot of logged safety audit events.
pub fn get_safety_events() -> Vec<SafetyEventRecord> {
    SAFETY_EVENTS.lock().map(|e| e.clone()).unwrap_or_default()
}
