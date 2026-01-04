use env_logger::fmt::{Color, Formatter};
use env_logger::Builder;
use log::{Level, LevelFilter, Record};
use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use log::info;

const SUCCESS_TINT: (u8, u8, u8) = (0, 255, 0); // pure green
const WARNING_TINT: (u8, u8, u8) = (255, 255, 0); // pure yellow

/// 50 / 50 blend of two RGB colors
#[inline]
fn blend((r1, g1, b1): (u8, u8, u8), (r2, g2, b2): (u8, u8, u8)) -> (u8, u8, u8) {
    (
        ((r1 as u16 + r2 as u16) / 2) as u8,
        ((g1 as u16 + g2 as u16) / 2) as u8,
        ((b1 as u16 + b2 as u16) / 2) as u8,
    )
}
/// Base color for each standard log level
#[inline]
fn base_rgb(level: Level) -> (u8, u8, u8) {
    match level {
        Level::Error => (255, 0, 0),     // red
        Level::Warn => (255, 255, 0),    // yellow
        Level::Info => (255, 255, 255),  // white
        Level::Debug => (200, 200, 255), // blue
        Level::Trace => (220, 220, 220), // grey
    }
}

pub fn init_log() -> anyhow::Result<()> {
    Builder::new()
        .format(|buf: &mut Formatter, record: &Record| {
            // ───── COLOUR  ────────────────────────────────────────────────────────
            let mut style = buf.style();
            let rgb = match record.target() {
                "success" => blend(base_rgb(record.level()), SUCCESS_TINT),
                "warning" => blend(base_rgb(record.level()), WARNING_TINT),
                _ => base_rgb(record.level()),
            };
            style.set_color(Color::Rgb(rgb.0, rgb.1, rgb.2));

            match record.level() {
                Level::Error | Level::Warn => style.set_bold(true),
                Level::Trace => style.set_dimmed(true),
                _ => style.set_bold(false),
            };

            // ───── CLICKABLE  src/…/file.rs:line:1  LINK ─────────────────────────
            // 1.  absolute path → strip project root → relative
            let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
            let mut full = PathBuf::from(record.file().unwrap_or("unknown"));
            if full.is_relative() {
                full = env::current_dir().unwrap_or_default().join(full);
            }
            let rel = full
                .strip_prefix(manifest_dir)
                .unwrap_or(&full) // fall back to abs
                .to_string_lossy()
                .replace('\\', "/"); // Windows → forward-slash

            // 2.  compiler pattern:  --> src/foo.rs:42:1
            let line = record.line().unwrap_or(0);
            let location = format!("\n{rel}:{line}:1");

            // ───── FINAL WRITE-OUT  (path part has *no* ANSI codes) ──────────────
            writeln!(
                buf,
                "[{} {}] {}  {}",
                chrono::Local::now().format("%H:%M:%S"),
                style.value(record.level()),
                style.value(record.args()),
                location // JetBrains makes this blue & clickable
            )
        })
        .filter_level(LevelFilter::Trace)
        .try_init() // ignore "already initialised" error
        .map_err(Into::into)
}

#[macro_export]
macro_rules! info_success {
    ($($arg:tt)*) => {
        ::log::info!(target: "success", "[SUCCESS] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! debug_success {
    ($($arg:tt)*) => {
        ::log::debug!(target: "success", "[SUCCESS] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! trace_success {
    ($($arg:tt)*) => {
        ::log::trace!(target: "success", "[SUCCESS] {}", format!($($arg)*));
    };
}


#[macro_export]
macro_rules! trace_warning
{ ($($arg:tt)*) => { ::log::trace!(target: "warning",  "[WARNING] {}",  format!($($arg)*)); }; }
#[macro_export]
macro_rules! debug_warning
{ ($($arg:tt)*) => { ::log::debug!(target: "warning",  "[WARNING] {}",  format!($($arg)*)); }; }
#[macro_export]
macro_rules! info_warning
{ ($($arg:tt)*) => { ::log::info! (target: "warning",  "[WARNING] {}",  format!($($arg)*)); }; }
#[macro_export]
macro_rules! warn_warning
{ ($($arg:tt)*) => { ::log::warn! (target: "warning",  "[WARNING] {}",  format!($($arg)*)); }; }
