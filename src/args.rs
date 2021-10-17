use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "MinecraftCompose", about = "Manage minecraft servers")]
pub struct Args {
    #[structopt(
        short,
        long,
        value_name = "FILE",
        help = "Sets the file to use",
        default_value = "./minecraft-compose.toml"
    )]
    pub file: String,

    #[structopt(
        short,
        long,
        help = "Enables extremely verbose debug output",
        hidden = !cfg!(debug_assertions)
    )]
    pub debug: bool,

    #[structopt(short, long, help = "Silences all output except errors")]
    pub quiet: bool,

    #[structopt(
        short = "v",
        long = "verbose",
        parse(from_occurrences),
        help = "Prints additional output"
    )]
    pub verbosity: u64,

    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    #[structopt(about = "Creates and starts the server container")]
    Up,

    #[structopt(about = "Stops and destroys the server container")]
    Down,

    #[structopt(about = "Creates the server container")]
    Create,

    #[structopt(about = "Destroys the server container")]
    Destroy,

    #[structopt(about = "Starts the server container")]
    Start,

    #[structopt(about = "Stops the server container")]
    Stop,

    #[structopt(about = "Displays the container status")]
    Status,

    #[structopt(about = "Connects a console to the server")]
    Console,

    #[structopt(about = "Manage datapacks for the server")]
    Datapacks(DatapackCommand),
}

#[derive(Debug, StructOpt)]
pub enum DatapackCommand {
    #[structopt(about = "Syncs datapacks to the server")]
    Sync,
}
