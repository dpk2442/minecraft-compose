use clap::{App, Arg, SubCommand};

mod config;
mod logging;
mod providers;
mod subcommands;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
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
        .subcommand(SubCommand::with_name("up").about("Creates and starts the server container"))
        .subcommand(SubCommand::with_name("down").about("Stops and destroys the server container"))
        .subcommand(SubCommand::with_name("create").about("Creates the server container"))
        .subcommand(SubCommand::with_name("destroy").about("Destroys the server container"))
        .subcommand(SubCommand::with_name("start").about("Starts the server container"))
        .subcommand(SubCommand::with_name("stop").about("Stops the server container"))
        .setting(clap::AppSettings::SubcommandRequired)
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

    if let Some(parent_dir) = std::path::Path::new(file_path).parent() {
        log::trace!(
            "Changing to config file directory: {}",
            parent_dir.display()
        );
        match std::env::set_current_dir(parent_dir) {
            Ok(_) => (),
            Err(err) => {
                log::error!("Unable to change to config file directory: {}", err);
                std::process::exit(1);
            }
        }
    }

    log::debug!(
        "Running from the directory {}",
        std::env::current_dir().unwrap().display()
    );

    let subcommands = match subcommands::new_from_defaults() {
        Ok(subcommands) => subcommands,
        Err(_) => {
            log::error!("Encountered an unexpected error");
            std::process::exit(1);
        }
    };

    let _ = match matches.subcommand() {
        ("up", Some(_)) => subcommands.up(&config),
        ("down", Some(_)) => subcommands.down(&config),
        ("create", Some(_)) => subcommands.create(&config),
        ("destroy", Some(_)) => subcommands.destroy(&config),
        ("start", Some(_)) => subcommands.start(&config),
        ("stop", Some(_)) => subcommands.stop(&config),
        _ => Ok(()),
    };
}
