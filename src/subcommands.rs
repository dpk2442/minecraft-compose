use crate::config;
use crate::providers;

pub struct SubCommands<T: providers::container::ContainerProvider> {
    container_provider: T,
}

pub fn new_from_defaults() -> Result<
    SubCommands<
        providers::container::ContainerProviderImpl<providers::backends::docker::DockerBackendImpl>,
    >,
    (),
> {
    Ok(SubCommands {
        container_provider: providers::container::new_from_defaults()?,
    })
}

impl<'a, T: providers::container::ContainerProvider> SubCommands<T> {
    pub fn up(&self, config: &config::Config, args: &clap::ArgMatches<'a>) {
        log::info!("Running up. config={:?}, args={:?}", config, args);
        log::info!(
            "Docker version: {:?}",
            self.container_provider.get_version()
        );
    }

    pub fn down(&self, config: &config::Config, args: &clap::ArgMatches<'a>) {
        log::info!("Running down. config={:?}, args={:?}", config, args);
        log::info!(
            "Docker version: {:?}",
            self.container_provider.get_version()
        );
    }
}
