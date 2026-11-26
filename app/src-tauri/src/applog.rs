// SPDX-License-Identifier: GPL-3.0-or-later

//! The debug log (ADR-0029): one plain-text file, flushed per line so a crash loses nothing,
//! rotated at 2 MB at startup (one old generation kept), with session markers; an in-memory
//! ring buffer for the Log viewer; a panic hook; and the session lock's timing and watchdog
//! ([`SessionLock`]). Records never hold document contents or secrets (callers log names,
//! ids and counts; [`redact`] masks tokens and keys that slip into a message).
//!
//! Line format: `2026-11-26T10:07:03.406 WARN  ui.stall  the window was held 4210 ms (…)`;
//! a multi-line message continues on lines indented by two spaces.

use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Serialize;

/// Past this size at startup the log becomes `cruncha.old.log`.
pub const MAX_BYTES: u64 = 2 * 1024 * 1024;
/// Records kept in memory for the viewer.
pub const RING: usize = 5000;
const FILE: &str = "cruncha.log";
const OLD: &str = "cruncha.old.log";
const STOPPED: &str = "=== stopped ===";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
}

impl Level {
    fn label(self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
        }
    }

    pub fn parse(s: &str) -> Option<Level> {
        match s.to_ascii_lowercase().as_str() {
            "error" => Some(Level::Error),
            "warn" | "warning" => Some(Level::Warn),
            "info" => Some(Level::Info),
            "debug" => Some(Level::Debug),
            _ => None,
        }
    }
}

/// One record, as the viewer shows it.
#[derive(Debug, Clone, Serialize)]
pub struct Record {
    pub seq: u64,
    /// Local time to the millisecond.
    pub at: String,
    pub level: Level,
    pub category: String,
    pub message: String,
}

impl Record {
    fn line(&self) -> String {
        let msg = self.message.replace('\n', "\n  ");
        format!("{} {:<5} {:<9} {}\n", self.at, self.level.label(), self.category, msg)
    }
}

type Sink = Box<dyn Fn(&Record) + Send + Sync>;

struct Log {
    file: Mutex<Option<(File, PathBuf)>>,
    ring: Mutex<VecDeque<Record>>,
    seq: AtomicU64,
    /// Records at this level or more important go to the file (all go to the ring).
    file_level: Level,
    sink: Mutex<Option<Sink>>,
}

fn global() -> &'static Log {
    static LOG: OnceLock<Log> = OnceLock::new();
    LOG.get_or_init(|| Log {
        file: Mutex::new(None),
        ring: Mutex::new(VecDeque::with_capacity(RING)),
        seq: AtomicU64::new(0),
        file_level: std::env::var("CRUNCHA_LOG").ok().and_then(|s| Level::parse(&s)).unwrap_or(Level::Info),
        sink: Mutex::new(None),
    })
}

fn now() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f").to_string()
}

/// Mask what looks like a secret: bearer tokens, API keys (`sk-…`), `token=`/`key=` values.
pub fn redact(msg: &str) -> String {
    let mut out = String::with_capacity(msg.len());
    let words: Vec<&str> = msg.split(' ').collect();
    let mut hide_next = false;
    for (i, w) in words.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let lower = w.to_ascii_lowercase();
        if hide_next {
            out.push_str("[redacted]");
            hide_next = false;
        } else if lower == "bearer" || lower.ends_with("authorization:") {
            out.push_str(w);
            // "Authorization: Bearer <token>": the token, not the scheme, is hidden.
            hide_next = !words.get(i + 1).is_some_and(|n| n.eq_ignore_ascii_case("bearer"));
        } else if w.starts_with("sk-") && w.len() > 12 {
            out.push_str("[redacted]");
        } else if let Some(eq) = w.find('=')
            && ["token", "key", "secret", "password", "api_key", "apikey"].contains(&lower[..eq].trim_start_matches(['"', '\'']))
        {
            out.push_str(&w[..=eq]);
            out.push_str("[redacted]");
        } else {
            out.push_str(w);
        }
    }
    out
}

/// Record a line.
pub fn log(level: Level, category: &str, message: impl AsRef<str>) {
    let g = global();
    let r = Record {
        seq: g.seq.fetch_add(1, Ordering::Relaxed) + 1,
        at: now(),
        level,
        category: category.to_owned(),
        message: redact(message.as_ref().trim_end()),
    };
    if level <= g.file_level
        && let Ok(mut f) = g.file.lock()
        && let Some((file, _)) = f.as_mut()
    {
        let _ = file.write_all(r.line().as_bytes());
        let _ = file.flush();
    }
    if let Ok(mut ring) = g.ring.lock() {
        if ring.len() == RING {
            ring.pop_front();
        }
        ring.push_back(r.clone());
    }
    if let Ok(sink) = g.sink.lock()
        && let Some(s) = sink.as_ref()
    {
        s(&r);
    }
}

pub fn error(category: &str, message: impl AsRef<str>) {
    log(Level::Error, category, message)
}
pub fn warn(category: &str, message: impl AsRef<str>) {
    log(Level::Warn, category, message)
}
pub fn info(category: &str, message: impl AsRef<str>) {
    log(Level::Info, category, message)
}
pub fn debug(category: &str, message: impl AsRef<str>) {
    log(Level::Debug, category, message)
}

/// Write a raw line to the file only (the session markers).
fn raw(line: &str) {
    if let Ok(mut f) = global().file.lock()
        && let Some((file, _)) = f.as_mut()
    {
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
}

/// Open the log in `dir`: rotate it past [`MAX_BYTES`], note whether the last session ended
/// without its stop marker (a crash or a kill), write this session's marker, and install the
/// panic hook. Returns the file's path.
pub fn init(dir: &Path, version: &str) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(FILE);
    let crashed = last_session_crashed(&path);
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > MAX_BYTES) {
        std::fs::rename(&path, dir.join(OLD))?;
    }
    let file = OpenOptions::new().create(true).append(true).open(&path)?;
    *global().file.lock().unwrap() = Some((file, path.clone()));
    raw(&format!("=== Cruncha {version} started {} (pid {}) ===", now(), std::process::id()));
    if crashed {
        warn("app", "the previous session ended without stopping (a crash, a freeze stopped with Ctrl-C, or a kill); its last lines are under Earlier sessions");
    }
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current().name().unwrap_or("unnamed").to_owned();
        let at = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        let msg = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_default();
        let bt = std::backtrace::Backtrace::force_capture();
        error("app", format!("panic in thread {thread} at {at}: {msg}\n{bt}"));
        default(info);
    }));
    Ok(path)
}

/// Whether the file's last session has a start marker and no stop marker after it.
fn last_session_crashed(path: &Path) -> bool {
    let Ok(text) = read_tail(path, 64 * 1024) else { return false };
    let last_start = text.rfind("=== Cruncha ");
    let last_stop = text.rfind(STOPPED);
    match (last_start, last_stop) {
        (Some(a), Some(b)) => b < a,
        (Some(_), None) => true,
        _ => false,
    }
}

/// The session's stop marker (a clean quit).
pub fn stopped() {
    info("app", "quitting");
    raw(STOPPED);
}

/// The last `bytes` of the file (from a line start), for "Earlier sessions".
pub fn read_tail(path: &Path, bytes: u64) -> std::io::Result<String> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = File::open(path)?;
    let len = f.metadata()?.len();
    let start = len.saturating_sub(bytes);
    f.seek(SeekFrom::Start(start))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let text = String::from_utf8_lossy(&buf).into_owned();
    Ok(if start > 0 { text.split_once('\n').map(|(_, r)| r.to_owned()).unwrap_or(text) } else { text })
}

/// The log file's path, once [`init`] ran.
pub fn path() -> Option<PathBuf> {
    global().file.lock().ok().and_then(|f| f.as_ref().map(|(_, p)| p.clone()))
}

/// Records after `since` (0: all in memory).
pub fn records(since: u64) -> Vec<Record> {
    global().ring.lock().map(|r| r.iter().filter(|x| x.seq > since).cloned().collect()).unwrap_or_default()
}

/// Call `f` with each new record (the app emits them to the page).
pub fn on_record(f: impl Fn(&Record) + Send + Sync + 'static) {
    *global().sink.lock().unwrap() = Some(Box::new(f));
}

/// An operation's line (ADR-0029 §2): logged at INFO with its time when it ends (dropped).
pub struct Op {
    category: &'static str,
    what: String,
    start: Instant,
}

impl Op {
    pub fn new(category: &'static str, what: impl Into<String>) -> Op {
        let what = what.into();
        debug(category, format!("{what}…"));
        Op { category, what, start: Instant::now() }
    }
}

impl Drop for Op {
    fn drop(&mut self) {
        info(self.category, format!("{} ({} ms)", self.what, self.start.elapsed().as_millis()));
    }
}

// ---------------------------------------------------------------------------------------
// Dependencies' `tracing` events at WARN and above join the log (category: their target's crate).

struct Bridge;

struct Message(String);
impl tracing::field::Visit for Message {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        } else {
            self.0.push_str(&format!(" {}={value:?}", field.name()));
        }
    }
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Bridge {
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        let level = match *event.metadata().level() {
            tracing::Level::ERROR => Level::Error,
            tracing::Level::WARN => Level::Warn,
            _ => return,
        };
        let mut m = Message(String::new());
        event.record(&mut m);
        let target = event.metadata().target().split("::").next().unwrap_or("dep");
        log(level, target, m.0);
    }
}

/// Route dependencies' `tracing` warnings and errors into the log (once).
pub fn bridge_tracing() {
    use tracing_subscriber::layer::SubscriberExt;
    let _ = tracing::subscriber::set_global_default(tracing_subscriber::registry().with(Bridge));
}

// ---------------------------------------------------------------------------------------
// The session lock (ADR-0029 §2): who holds it, who waits, how long; slow commands logged.

/// Commands slower than this are logged at INFO (`CRUNCHA_SLOW_MS`; 0 logs every one).
pub fn slow_ms() -> u64 {
    static MS: OnceLock<u64> = OnceLock::new();
    *MS.get_or_init(|| std::env::var("CRUNCHA_SLOW_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(100))
}

#[derive(Default)]
struct Holders {
    holder: Option<(&'static str, Instant)>,
    waiting: Vec<(u64, &'static str, Instant)>,
    next: u64,
    /// When the watchdog last reported the current holder.
    reported: Option<Instant>,
}

/// The app's `Session` behind a lock that knows which command holds it (ADR-0029).
pub struct SessionLock<T> {
    inner: tokio::sync::Mutex<T>,
    holders: Arc<Mutex<Holders>>,
}

/// A held session: derefs to it; logs the command's time when dropped.
pub struct Guard<'a, T> {
    guard: tokio::sync::MutexGuard<'a, T>,
    holders: Arc<Mutex<Holders>>,
    what: &'static str,
    waited: Duration,
    since: Instant,
}

impl<T> std::ops::Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.guard
    }
}
impl<T> std::ops::DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.guard
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        let held = self.since.elapsed();
        if let Ok(mut h) = self.holders.lock() {
            h.holder = None;
            h.reported = None;
        }
        let total = (held + self.waited).as_millis() as u64;
        let line = format!("{} took {} ms ({} ms waiting for the session)", self.what, total, self.waited.as_millis());
        if total >= slow_ms() {
            info("cmd.slow", line);
        } else {
            debug("cmd", line);
        }
    }
}

impl<T> SessionLock<T> {
    pub fn new(value: T) -> Self {
        SessionLock { inner: tokio::sync::Mutex::new(value), holders: Arc::new(Mutex::new(Holders::default())) }
    }

    /// Take the session for `what` (the command or operation's name).
    pub async fn lock(&self, what: &'static str) -> Guard<'_, T> {
        let start = Instant::now();
        let ticket = {
            let mut h = self.holders.lock().unwrap();
            h.next += 1;
            let t = h.next;
            h.waiting.push((t, what, start));
            t
        };
        let guard = self.inner.lock().await;
        let since = Instant::now();
        {
            let mut h = self.holders.lock().unwrap();
            h.waiting.retain(|(t, ..)| *t != ticket);
            h.holder = Some((what, since));
            h.reported = None;
        }
        Guard { guard, holders: self.holders.clone(), what, waited: since - start, since }
    }

    /// Start the watchdog: once a second, a command holding the lock 5 s or more is logged
    /// (WARN `lock`) with what waits behind it, and again every 10 s while it holds on.
    pub fn watch(&self) {
        let holders = self.holders.clone();
        std::thread::Builder::new()
            .name("cruncha-lock-watchdog".into())
            .spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                    check(&holders, Duration::from_secs(5), Duration::from_secs(10));
                }
            })
            .expect("the watchdog thread starts");
    }

    #[cfg(test)]
    fn check_now(&self, after: Duration) {
        check(&self.holders, after, Duration::from_secs(10));
    }
}

fn check(holders: &Mutex<Holders>, after: Duration, every: Duration) {
    let Ok(mut h) = holders.lock() else { return };
    let Some((what, since)) = h.holder else { return };
    let held = since.elapsed();
    if held < after || h.reported.is_some_and(|r| r.elapsed() < every) {
        return;
    }
    h.reported = Some(Instant::now());
    let waiting: Vec<String> =
        h.waiting.iter().map(|(_, w, at)| format!("{w} ({:.1} s)", at.elapsed().as_secs_f64())).collect();
    warn(
        "lock",
        format!(
            "{what} has held the session for {:.1} s{}",
            held.as_secs_f64(),
            if waiting.is_empty() { String::new() } else { format!("; waiting: {}", waiting.join(", ")) }
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_are_masked() {
        assert_eq!(redact("Authorization: Bearer abc123 ok"), "Authorization: Bearer [redacted] ok");
        assert_eq!(redact("key sk-ant-api03-abcdefghijk"), "key [redacted]");
        assert_eq!(redact("url?x=1 token=abc api_key=zzz"), "url?x=1 token=[redacted] api_key=[redacted]");
        assert_eq!(redact("Insert template bolts 1.1.0 (19 cells)"), "Insert template bolts 1.1.0 (19 cells)");
    }

    #[test]
    fn a_session_without_a_stop_marker_crashed() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join(FILE);
        std::fs::write(&p, "=== Cruncha 0.1 started x (pid 1) ===\nline\n=== stopped ===\n").unwrap();
        assert!(!last_session_crashed(&p));
        std::fs::write(&p, "=== Cruncha 0.1 started x (pid 1) ===\nline\n=== stopped ===\n=== Cruncha 0.1 started y (pid 2) ===\nlast words\n").unwrap();
        assert!(last_session_crashed(&p));
    }

    #[test]
    fn records_have_their_format_and_reach_the_ring() {
        let r = Record { seq: 1, at: "2026-11-26T10:07:03.406".into(), level: Level::Warn, category: "ui.stall".into(), message: "held\nfor long".into() };
        assert_eq!(r.line(), "2026-11-26T10:07:03.406 WARN  ui.stall  held\n  for long\n");
        info("test", "hello ring");
        assert!(records(0).iter().any(|r| r.category == "test" && r.message == "hello ring"));
    }

    #[test]
    fn the_tail_starts_at_a_line() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("x.log");
        std::fs::write(&p, "first line\nsecond line\nthird\n").unwrap();
        assert_eq!(read_tail(&p, 14).unwrap(), "third\n");
        assert_eq!(read_tail(&p, 1000).unwrap(), "first line\nsecond line\nthird\n");
    }

    #[tokio::test]
    async fn the_watchdog_names_the_holder_and_those_waiting() {
        let lock = Arc::new(SessionLock::new(0u32));
        let g = lock.lock("insert_template").await;
        let waiter = lock.clone();
        let w = tokio::spawn(async move {
            let _g = waiter.lock("results").await;
        });
        tokio::time::sleep(Duration::from_millis(50)).await;
        lock.check_now(Duration::ZERO);
        let found = records(0).into_iter().rev().find(|r| r.category == "lock").expect("a lock record");
        assert!(found.message.starts_with("insert_template has held the session for"), "{}", found.message);
        assert!(found.message.contains("waiting: results"), "{}", found.message);
        drop(g);
        w.await.unwrap();
        assert!(records(0).iter().any(|r| r.message.starts_with("insert_template took")));
    }
}
