use log::{Level, Metadata, Record};

struct ConsoleLogger {
    level: Level,
}

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        let metadata = record.metadata();
        if self.enabled(metadata) {
            match metadata.level() {
                Level::Error | Level::Warn => eprintln!("{}", record.args()),
                _ => println!("{}", record.args()),
            }
        }
    }

    fn flush(&self) {}
}

fn get_level(quiet: bool, verbosity: u64) -> Level {
    if quiet {
        Level::Error
    } else {
        match verbosity {
            0 => Level::Warn,
            1 => Level::Info,
            2 => Level::Debug,
            _ => Level::Trace,
        }
    }
}

pub fn init_logging(quiet: bool, verbosity: u64) -> Result<(), log::SetLoggerError> {
    let level = get_level(quiet, verbosity);
    log::set_boxed_logger(Box::new(ConsoleLogger { level: level }))?;
    log::set_max_level(log::LevelFilter::max());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! get_level_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (quiet, verbosity, expected) = $value;
                assert_eq!(expected, get_level(quiet, verbosity));
            }
        )*
        }
    }

    get_level_tests! {
        get_level_quiet: (true, 0, Level::Error),
        get_level_no_args: (false, 0, Level::Warn),
        get_level_v: (false, 1, Level::Info),
        get_level_vv: (false, 2, Level::Debug),
        get_level_vvv: (false, 3, Level::Trace),
    }
}
