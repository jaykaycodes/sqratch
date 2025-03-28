use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};
use time;

#[cfg(not(debug_assertions))]
use log::Level;

fn setup_logging_plugin() -> tauri_plugin_log::Builder {
    let builder = tauri_plugin_log::Builder::new()
        .clear_targets()
        .max_file_size(2_000_000)
        .rotation_strategy(RotationStrategy::KeepAll)
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
    let builder = builder.target(Target::new(TargetKind::LogDir { file_name: None }).filter(
        |metadata| {
            metadata.level() != Level::Debug
                && metadata.level() != Level::Trace
                && !metadata.target().starts_with("tao")
        },
    ));

    builder
}
