use ansi_term::Color;
use log::{Level, SetLoggerError};

pub struct MainLogger;

impl log::Log for MainLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &log::Record) {
        match record.level() {
            Level::Trace => {
                println!("{}", record.args());
            }
            Level::Info => {
                println!("{} {}", Color::Blue.paint("[INFO]"), record.args())
            }
            Level::Warn => {
                println!(
                    "{} {}",
                    Color::Yellow.paint("[WARNING]"),
                    record.args()
                )
            }
            Level::Error => {
                println!("{} {}", Color::Red.paint("[ERROR]"), record.args())
            }
            _ => (),
        }
    }

    fn flush(&self) {}
}

impl MainLogger {
    pub fn set_as_logger() -> Result<(), SetLoggerError> {
        let main_logger = Box::new(MainLogger);
        log::set_boxed_logger(main_logger)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
    }
}
