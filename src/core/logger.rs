use std::io::Write;
use chrono::Local;

pub fn init_logger() {
    env_logger::builder()
        .format(|buf, record| {
            let time_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

            let level_style = buf.default_level_style(record.level());

            writeln!(
                buf,
                "[{}][{}{}{:#}][{}] {}",
                time_str,
                level_style,
                record.level(),
                level_style,
                record.module_path().unwrap_or("未知"),
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Trace)
        .init();
}