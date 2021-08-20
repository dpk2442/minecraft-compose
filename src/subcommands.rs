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
        let version_result = self.container_provider.get_version();
        log::info!("Docker version: {:?}", version_result);
    }

    pub fn down(&self, config: &config::Config, args: &clap::ArgMatches<'a>) {
        log::info!("Running down. config={:?}, args={:?}", config, args);
        log::info!(
            "Docker version: {:?}",
            self.container_provider.get_version()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::container::MockContainerProvider;

    fn get_subcommands() -> SubCommands<MockContainerProvider> {
        SubCommands {
            container_provider: MockContainerProvider::new(),
        }
    }

    fn get_config() -> config::Config {
        config::Config {
            name: "name".to_owned(),
            host: "0.0.0.0".to_owned(),
            port: 25565,
        }
    }

    fn get_arg_matches<'a>() -> clap::ArgMatches<'a> {
        clap::ArgMatches {
            args: std::collections::HashMap::new(),
            subcommand: None,
            usage: None,
        }
    }

    #[test]
    fn test_up_calls_version() {
        let mut subcommands = get_subcommands();
        let config = get_config();
        let args = get_arg_matches();

        subcommands
            .container_provider
            .expect_get_version()
            .times(1)
            .returning(|| Ok("test version".to_owned()));

        subcommands.up(&config, &args);
    }
}
