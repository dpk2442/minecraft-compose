use clap::{App, Arg};

mod config;

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
        .get_matches();

    let file_path = matches.value_of("file").unwrap_or("./mcc.toml");
    let config = match config::load_config(file_path) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Unable to load config file: {}", err);
            std::process::exit(1);
        }
    };

    println!("Loaded config: {:?}", config);
}
