use ansi_term::Color;
use log::Level;

pub struct StatusLogger;

impl log::Log for StatusLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &log::Record) {
        println!("{}", record.args())
    }

    fn flush(&self) {}
}

impl StatusLogger {
    pub fn new_file(path: &str) {
        let indent = 4usize;
        println!(
            "{:indent$}{} {}",
            "",
            Color::Green.paint("[new file]"),
            path
        );
    }

    pub fn modified_file(path: &str) {
        let indent = 4usize;
        println!(
            "{:indent$}{} {}",
            "",
            Color::Yellow.paint("[modified]"),
            path
        );
    }

    pub fn modified_unsaved_file(path: &str) {
        let indent = 4usize;
        println!(
            "{:indent$}{}{} {}",
            "",
            Color::Yellow.paint("[modified]"),
            Color::Red.paint("(unsaved)"),
            path
        );
    }

    pub fn untracked_file(path: &str) {
        let indent = 4usize;
        println!("{:indent$}{} {}", "", Color::Red.paint("[untracked]"), path);
    }

    pub fn removed_file(path: &str) {
        let indent = 4usize;
        println!("{:indent$}{} {}", "", Color::Red.paint("[removed]"), path);
    }

    pub fn removed_unsaved_file(path: &str) {
        let indent = 4usize;
        println!(
            "{:indent$}{}{} {}",
            "",
            Color::Red.paint("[removed]"),
            Color::Red.paint("(unsaved)"),
            path
        );
    }
}
