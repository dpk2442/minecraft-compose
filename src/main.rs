use structopt::StructOpt;

mod args;
mod config;
mod logging;
mod providers;
mod subcommands;

#[tokio::main]
async fn main() {
    let args = args::Args::from_args();

    match logging::init_logging(args.debug, args.quiet, args.verbosity) {
        Err(err) => {
            eprintln!("Unable to initialize logging: {}", err);
            std::process::exit(1);
        }
        _ => (),
    }

    let config = match config::load_config(&args.file) {
        Ok(config) => config,
        Err(err) => {
            log::error!("Unable to load config file: {}", err);
            std::process::exit(1);
        }
    };

    if let Some(parent_dir) = std::path::Path::new(&args.file).parent() {
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

    log::trace!(
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

    let _ = match args.subcommand {
        args::SubCommand::Up => subcommands.up(&config),
        args::SubCommand::Down => subcommands.down(&config),
        args::SubCommand::Create => subcommands.create(&config),
        args::SubCommand::Destroy => subcommands.destroy(&config),
        args::SubCommand::Start => subcommands.start(&config),
        args::SubCommand::Stop => subcommands.stop(&config),
        args::SubCommand::Status => subcommands.status(&config),
        args::SubCommand::Console => subcommands.console(&config),
        args::SubCommand::Datapacks(args::DatapackCommand::Sync) => {
            subcommands.sync_datapacks(&config)
        }
    };
}
