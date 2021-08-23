use crate::config;
use crate::providers;

pub struct SubCommands<
    T1: providers::container::ContainerProvider,
    T2: providers::file::FileProvider,
> {
    container_provider: T1,
    file_provider: T2,
}

pub fn new_from_defaults() -> Result<
    SubCommands<
        providers::container::ContainerProviderImpl<providers::backends::docker::DockerBackendImpl>,
        providers::file::FileProviderImpl<providers::backends::filesystem::FilesystemBackendImpl>,
    >,
    (),
> {
    Ok(SubCommands {
        container_provider: providers::container::new_from_defaults()?,
        file_provider: providers::file::new_from_defaults(),
    })
}

impl<'a, T1: providers::container::ContainerProvider, T2: providers::file::FileProvider>
    SubCommands<T1, T2>
{
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

    pub fn create(&self, config: &config::Config) {
        if let Err(()) = self
            .container_provider
            .create_container(config, self.file_provider.get_data_path())
        {
            log::error!("Failed to created container");
            return;
        }
    }

    pub fn start(&self, _config: &config::Config) {
        if let Err(()) = self.file_provider.create_data_folder() {
            log::error!("Failed to create data folder");
            return;
        }

        if let Err(()) = self.file_provider.create_and_populate_server_properties() {
            log::error!("Failed to create server.properties");
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use super::*;
    use crate::providers::container::MockContainerProvider;
    use crate::providers::file::MockFileProvider;

    fn get_subcommands() -> SubCommands<MockContainerProvider, MockFileProvider> {
        SubCommands {
            container_provider: MockContainerProvider::new(),
            file_provider: MockFileProvider::new(),
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

    #[test]
    fn test_create() {
        let mut subcommands = get_subcommands();
        let config = get_config();
        let path = std::path::PathBuf::new();
        let path_clone = path.clone();

        subcommands
            .file_provider
            .expect_get_data_path()
            .times(1)
            .return_const(path);

        subcommands
            .container_provider
            .expect_create_container()
            .with(eq(config.clone()), eq(path_clone))
            .returning(|_, _| Ok(()));

        subcommands.create(&config);
    }

    #[test]
    fn test_start() {
        let mut subcommands = get_subcommands();
        let config = get_config();

        subcommands
            .file_provider
            .expect_create_data_folder()
            .times(1)
            .returning(|| Ok(()));

        subcommands
            .file_provider
            .expect_create_and_populate_server_properties()
            .times(1)
            .returning(|| Ok(()));

        subcommands.start(&config);
    }
}
