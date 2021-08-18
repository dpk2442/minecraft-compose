use clap::{App, Arg};

mod config;
mod logging;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let matches = App::new("MinecraftCompose")
        .about("Manage minecraft servers")
        .version(VERSION)
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("Sets the file to use, defaults to ./mcc.toml")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Silences all output except errors"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Prints additional output"),
        )
        .get_matches();

    let quiet = matches.is_present("quiet");
    let verbosity = if matches.is_present("verbose") {
        matches.occurrences_of("verbose")
    } else {
        0
    };
    match logging::init_logging(quiet, verbosity) {
        Err(err) => {
            eprintln!("Unable to initialize logging: {}", err);
            std::process::exit(1);
        }
        _ => (),
    }

    let file_path = matches.value_of("file").unwrap_or("./mcc.toml");
    let config = match config::load_config(file_path) {
        Ok(config) => config,
        Err(err) => {
            log::error!("Unable to load config file: {}", err);
            std::process::exit(1);
        }
    };

    println!("Loaded config: {:?}", config);
}
