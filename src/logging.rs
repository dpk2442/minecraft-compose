use log::{Level, Log, Metadata, Record};

const CRATE_NAME: &'static str = env!("CARGO_CRATE_NAME");

struct ConsoleLogger {
    debug: bool,
    level: Level,
}

impl Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        (self.debug || metadata.target().starts_with(CRATE_NAME)) && metadata.level() <= self.level
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

fn get_level(debug: bool, quiet: bool, verbosity: u64) -> Level {
    if debug {
        Level::Trace
    } else if quiet {
        Level::Error
    } else {
        match verbosity {
            0 => Level::Info,
            1 => Level::Debug,
            _ => Level::Trace,
        }
    }
}

pub fn init_logging(debug: bool, quiet: bool, verbosity: u64) -> Result<(), log::SetLoggerError> {
    let level = get_level(debug, quiet, verbosity);
    log::set_boxed_logger(Box::new(ConsoleLogger { debug, level }))?;
    log::set_max_level(log::LevelFilter::max());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::MetadataBuilder;

    macro_rules! get_level_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (debug, quiet, verbosity, expected) = $value;
                assert_eq!(expected, get_level(debug, quiet, verbosity));
            }
        )*
        }
    }

    get_level_tests! {
        get_level_quiet: (false, true, 0, Level::Error),
        get_level_no_args: (false, false, 0, Level::Info),
        get_level_v: (false, false, 1, Level::Debug),
        get_level_vv: (false, false, 2, Level::Trace),
        get_level_vvv: (false, false, 3, Level::Trace),
        get_level_debug: (true, false, 0, Level::Trace),
    }

    macro_rules! get_enabled_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (
                    debug,
                    level,
                    crate_info_expected,
                    crate_trace_expected,
                    external_info_expected,
                ) = $value;

                let logger = ConsoleLogger {
                    debug: debug,
                    level: level,
                };

                assert_eq!(
                    crate_info_expected,
                    logger.enabled(
                        &MetadataBuilder::new()
                            .level(Level::Info)
                            .target(&format!("{}", CRATE_NAME))
                            .build()
                    )
                );

                assert_eq!(
                    crate_trace_expected,
                    logger.enabled(
                        &MetadataBuilder::new()
                            .level(Level::Trace)
                            .target(&format!("{}", CRATE_NAME))
                            .build()
                    )
                );
            assert_eq!(
                external_info_expected,
                logger.enabled(
                    &MetadataBuilder::new()
                        .level(Level::Info)
                        .target("external_crate")
                        .build()
                )
            );
            }
        )*
        }
    }

    get_enabled_tests! {
        get_enabled_debug: (true, Level::Trace, true, true, true),
        get_enabled_no_debug_info: (false, Level::Info, true, false, false),
        get_enabled_no_debug_trace: (false, Level::Trace, true, true, false),
        get_enabled_no_debug_error: (false, Level::Error, false, false, false),
    }
}
