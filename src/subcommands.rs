use crate::config;

pub struct SubCommands;

impl SubCommands {
    pub fn up(&self, config: &config::Config, args: &clap::ArgMatches) {
        log::info!("Running up. config={:?}, args={:?}", config, args);
    }

    pub fn down(&self, config: &config::Config, args: &clap::ArgMatches) {
        log::info!("Running down. config={:?}, args={:?}", config, args);
    }
}
