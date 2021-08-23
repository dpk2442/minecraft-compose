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
    pub fn up(&self, config: &config::Config) -> Result<(), ()> {
        self.create(config)?;
        self.start(config)
    }

    pub fn down(&self, config: &config::Config) -> Result<(), ()> {
        self.stop(config)?;
        self.destroy(config)
    }

    pub fn create(&self, config: &config::Config) -> Result<(), ()> {
        if let Err(()) = self
            .container_provider
            .create_container(config, self.file_provider.get_data_path())
        {
            log::error!("Failed to create the container");
            return Err(());
        }

        Ok(())
    }

    pub fn destroy(&self, config: &config::Config) -> Result<(), ()> {
        if let Err(()) = self.container_provider.delete_container(&config) {
            log::error!("Failed to delete the container");
            return Err(());
        }

        Ok(())
    }

    pub fn start(&self, config: &config::Config) -> Result<(), ()> {
        if let Err(()) = self.file_provider.create_data_folder() {
            log::error!("Failed to create data folder");
            return Err(());
        }

        if let Err(()) = self.file_provider.create_and_populate_server_properties() {
            log::error!("Failed to create server.properties");
            return Err(());
        }

        if let Err(()) = self.container_provider.start_container(&config) {
            log::error!("Failed to start the container");
            return Err(());
        }

        Ok(())
    }

    pub fn stop(&self, config: &config::Config) -> Result<(), ()> {
        if let Err(()) = self.container_provider.stop_container(&config) {
            log::error!("Failed to start the container");
            return Err(());
        }

        Ok(())
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

        assert_eq!(Ok(()), subcommands.create(&config));
    }

    #[test]
    fn test_destroy() {
        let mut subcommands = get_subcommands();
        let config = get_config();

        subcommands
            .container_provider
            .expect_delete_container()
            .with(eq(config.clone()))
            .times(1)
            .returning(|_| Ok(()));

        assert_eq!(Ok(()), subcommands.destroy(&config));
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

        subcommands
            .container_provider
            .expect_start_container()
            .with(eq(config.clone()))
            .times(1)
            .returning(|_| Ok(()));

        assert_eq!(Ok(()), subcommands.start(&config));
    }

    #[test]
    fn test_stop() {
        let mut subcommands = get_subcommands();
        let config = get_config();

        subcommands
            .container_provider
            .expect_stop_container()
            .with(eq(config.clone()))
            .times(1)
            .returning(|_| Ok(()));

        assert_eq!(Ok(()), subcommands.stop(&config));
    }
}
