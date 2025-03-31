use log::LevelFilter;
use std::env;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};
use time;

const DEFAULT_LOG_LEVEL: LevelFilter = if cfg!(debug_assertions) {
    LevelFilter::Debug
} else {
    LevelFilter::Info
};

pub fn setup_logging_plugin() -> tauri_plugin_log::Builder {
    // Get log level from env, or use default based on debug/release mode
    let filter = env::var("LOG_LEVEL")
        .map(|level| match level.to_lowercase().as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => DEFAULT_LOG_LEVEL,
        })
        .unwrap_or(DEFAULT_LOG_LEVEL);

    let builder = tauri_plugin_log::Builder::new()
        .clear_targets()
        .max_file_size(2_000_000)
        .rotation_strategy(RotationStrategy::KeepAll)
        .level(filter)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                TimezoneStrategy::UseUtc
                    .get_now()
                    .format(
                        &time::format_description::parse(
                            "[year]-[month]-[day] [hour]:[minute]:[second]"
                        )
                        .unwrap()
                    )
                    .unwrap(),
                record.level(),
                message
            ))
        });

    #[cfg(debug_assertions)]
    let builder = builder.target(Target::new(TargetKind::Stdout));

    #[cfg(not(debug_assertions))]
    let builder = builder.target(
        Target::new(TargetKind::LogDir { file_name: None })
            .filter(|metadata| !metadata.target().starts_with("tao")),
    );

    builder
}
