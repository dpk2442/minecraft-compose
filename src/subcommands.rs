use crate::config;
use crate::providers::{
    self,
    container::{ContainerState, GameState},
};

pub struct SubCommands<
    T1: providers::container::ContainerProvider,
    T2: providers::file::FileProvider,
    T3: providers::game::GameProvider,
> {
    container_provider: T1,
    file_provider: T2,
    game_provider: T3,
}

pub fn new_from_defaults() -> Result<
    SubCommands<
        providers::container::ContainerProviderImpl<providers::backends::docker::DockerBackendImpl>,
        providers::file::FileProviderImpl<providers::backends::filesystem::FilesystemBackendImpl>,
        providers::game::GameProviderImpl<
            providers::backends::rcon::RconBackendFactoryImpl,
            providers::backends::input::InputBackendFactoryImpl,
        >,
    >,
    (),
> {
    Ok(SubCommands {
        container_provider: providers::container::new_from_defaults()?,
        file_provider: providers::file::new_from_defaults(),
        game_provider: providers::game::new_from_defaults(),
    })
}

impl<
        'a,
        T1: providers::container::ContainerProvider,
        T2: providers::file::FileProvider,
        T3: providers::game::GameProvider,
    > SubCommands<T1, T2, T3>
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
        if self.container_provider.get_container_status(&config)? != ContainerState::NotFound {
            log::warn!("Container already exists");
            return Ok(());
        }

        if let Err(()) = self.file_provider.create_data_folder() {
            log::error!("Failed to create data folder");
            return Err(());
        }

        let data_path = self.file_provider.get_data_path().or_else(|_| {
            log::error!("Failed to get the data path");
            Err(())
        })?;

        if let Err(()) = self.container_provider.create_container(config, &data_path) {
            log::error!("Failed to create the container");
            return Err(());
        }

        log::info!("Created the server container {}", config.name);
        Ok(())
    }

    pub fn destroy(&self, config: &config::Config) -> Result<(), ()> {
        if self.container_provider.get_container_status(&config)? != ContainerState::Stopped {
            log::error!("Container is not stopped");
            return Err(());
        }

        if let Err(()) = self.container_provider.delete_container(&config) {
            log::error!("Failed to delete the container");
            return Err(());
        }

        log::info!("Destroyed the server container {}", config.name);
        Ok(())
    }

    pub fn start(&self, config: &config::Config) -> Result<(), ()> {
        if self.container_provider.get_container_status(&config)? != ContainerState::Stopped {
            log::error!("Container is not stopped");
            return Err(());
        }

        if let Err(()) = self.file_provider.create_data_folder() {
            log::error!("Failed to create data folder");
            return Err(());
        }

        if let Err(()) = self
            .file_provider
            .create_and_populate_server_properties(&config)
        {
            log::error!("Failed to create server.properties");
            return Err(());
        }

        if let Err(()) = self.container_provider.start_container(&config) {
            log::error!("Failed to start the container");
            return Err(());
        }

        log::info!("Started the server container {}", config.name);
        Ok(())
    }

    pub fn stop(&self, config: &config::Config) -> Result<(), ()> {
        if !matches!(
            self.container_provider.get_container_status(&config)?,
            ContainerState::Running(_)
        ) {
            log::warn!("Container is not running");
            return Ok(());
        }

        if let Err(()) = self.container_provider.stop_container(&config) {
            log::error!("Failed to start the container");
            return Err(());
        }

        log::info!("Stopped the server container {}", config.name);
        Ok(())
    }

    pub fn status(&self, config: &config::Config) -> Result<(), ()> {
        match self.container_provider.get_container_status(&config) {
            Ok(status) => {
                match status {
                    providers::container::ContainerState::Unknown => {
                        log::info!("The state of the container is unkown");
                    }
                    providers::container::ContainerState::NotFound => {
                        log::info!("The container does not exist");
                    }
                    providers::container::ContainerState::Running(game_state) => {
                        log::info!("The container is currently running");
                        match game_state {
                            providers::container::GameState::Unknown => {
                                log::info!("The state of the server is unknown")
                            }
                            providers::container::GameState::Starting => {
                                log::info!("The server is starting")
                            }
                            providers::container::GameState::Running => {
                                log::info!("The server is running")
                            }
                        };
                    }
                    providers::container::ContainerState::Stopped => {
                        log::info!("The container is currently stopped");
                    }
                };
                Ok(())
            }
            Err(()) => {
                log::error!("Failed to get container status");
                Err(())
            }
        }
    }

    pub fn console(&self, config: &config::Config) -> Result<(), ()> {
        if self.container_provider.get_container_status(&config)?
            != ContainerState::Running(GameState::Running)
        {
            log::error!("Game server is not running");
            return Err(());
        }

        let (rcon_host, rcon_port) = self
            .container_provider
            .get_container_rcon_address(&config)
            .or_else(|_| {
                log::error!("Failed to get rcon address");
                return Err(());
            })?;

        self.game_provider
            .run_interactive_rcon_session(&config, &rcon_host, &rcon_port)
            .or_else(|_| {
                log::error!("Failed to establish interactive rcon session");
                return Err(());
            })?;

        Ok(())
    }

    pub fn sync_datapacks(&self, config: &config::Config) -> Result<(), ()> {
        if let Err(()) = self.file_provider.sync_datapacks(&config) {
            log::error!("Failed to sync datapacks");
            return Err(());
        }

        if self.container_provider.get_container_status(&config)?
            != ContainerState::Running(GameState::Running)
        {
            return Ok(());
        }

        log::info!("The game is running, attempting to reload datapacks");

        let (rcon_host, rcon_port) = self
            .container_provider
            .get_container_rcon_address(&config)
            .or_else(|_| {
                log::error!("Failed to get rcon address");
                return Err(());
            })?;

        self.game_provider
            .run_rcon_commands(
                &rcon_host,
                &rcon_port,
                vec!["reload".to_owned(), "datapack list".to_owned()],
            )
            .or_else(|_| {
                log::error!("Failed to run rcon commands");
                return Err(());
            })?
            .iter()
            .for_each(|response| println!("{:?}", response));

        Ok(())
    }

    pub fn logs(&self, config: &config::Config) -> Result<(), ()> {
        self.container_provider.display_container_logs(&config)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use super::*;
    use crate::providers::container::MockContainerProvider;
    use crate::providers::file::MockFileProvider;
    use crate::providers::game::MockGameProvider;

    fn get_subcommands() -> SubCommands<MockContainerProvider, MockFileProvider, MockGameProvider> {
        SubCommands {
            container_provider: MockContainerProvider::new(),
            file_provider: MockFileProvider::new(),
            game_provider: MockGameProvider::new(),
        }
    }

    fn get_config() -> config::Config {
        config::Config {
            name: "name".to_owned(),
            host: "0.0.0.0".to_owned(),
            port: 25565,
            server: config::Server {
                version: "1.17.1".to_owned(),
                server_type: config::ServerType::Vanilla,
                ..std::default::Default::default()
            },
            ..std::default::Default::default()
        }
    }

    #[test]
    fn test_create() {
        let mut subcommands = get_subcommands();
        let config = get_config();
        let path = std::path::PathBuf::new();
        let path_clone = path.clone();

        subcommands
            .container_provider
            .expect_get_container_status()
            .with(eq(config.clone()))
            .returning(|_| Ok(ContainerState::NotFound));

        subcommands
            .file_provider
            .expect_create_data_folder()
            .times(1)
            .return_const(Ok(()));

        subcommands
            .file_provider
            .expect_get_data_path()
            .times(1)
            .return_const(Ok(path));

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
            .expect_get_container_status()
            .with(eq(config.clone()))
            .returning(|_| Ok(ContainerState::Stopped));

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
            .container_provider
            .expect_get_container_status()
            .with(eq(config.clone()))
            .returning(|_| Ok(ContainerState::Stopped));

        subcommands
            .file_provider
            .expect_create_data_folder()
            .times(1)
            .returning(|| Ok(()));

        subcommands
            .file_provider
            .expect_create_and_populate_server_properties()
            .with(eq(config.clone()))
            .times(1)
            .returning(|_| Ok(()));

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
            .expect_get_container_status()
            .with(eq(config.clone()))
            .returning(|_| Ok(ContainerState::Running(GameState::Unknown)));

        subcommands
            .container_provider
            .expect_stop_container()
            .with(eq(config.clone()))
            .times(1)
            .returning(|_| Ok(()));

        assert_eq!(Ok(()), subcommands.stop(&config));
    }

    #[test]
    fn test_console() {
        let mut subcommands = get_subcommands();
        let config = get_config();

        subcommands
            .container_provider
            .expect_get_container_status()
            .with(eq(config.clone()))
            .returning(|_| Ok(ContainerState::Running(GameState::Running)));

        subcommands
            .container_provider
            .expect_get_container_rcon_address()
            .with(eq(config.clone()))
            .times(1)
            .returning(|_| Ok(("host".to_owned(), "port".to_owned())));

        subcommands
            .game_provider
            .expect_run_interactive_rcon_session()
            .with(eq(config.clone()), eq("host"), eq("port"))
            .times(1)
            .returning(|_, _, _| Ok(()));

        assert_eq!(Ok(()), subcommands.console(&config));
    }

    mod test_sync_datapacks {
        use super::*;

        #[test]
        fn game_not_running() {
            let config = get_config();
            let mut subcommands = get_subcommands();

            subcommands
                .file_provider
                .expect_sync_datapacks()
                .with(eq(config.clone()))
                .times(1)
                .returning(|_| Ok(()));

            subcommands
                .container_provider
                .expect_get_container_status()
                .with(eq(config.clone()))
                .times(1)
                .returning(|_| Ok(ContainerState::Stopped));

            assert_eq!(Ok(()), subcommands.sync_datapacks(&config));
        }

        #[test]
        fn game_running() {
            let config = get_config();
            let mut subcommands = get_subcommands();

            subcommands
                .file_provider
                .expect_sync_datapacks()
                .with(eq(config.clone()))
                .times(1)
                .returning(|_| Ok(()));

            subcommands
                .container_provider
                .expect_get_container_status()
                .with(eq(config.clone()))
                .times(1)
                .returning(|_| Ok(ContainerState::Running(GameState::Running)));

            subcommands
                .container_provider
                .expect_get_container_rcon_address()
                .with(eq(config.clone()))
                .times(1)
                .returning(|_| Ok(("host".to_owned(), "port".to_owned())));

            subcommands
                .game_provider
                .expect_run_rcon_commands()
                .with(
                    eq("host"),
                    eq("port"),
                    eq(vec!["reload".to_owned(), "datapack list".to_owned()]),
                )
                .times(1)
                .returning(|_, _, _| Ok(vec!["response".to_owned()]));

            assert_eq!(Ok(()), subcommands.sync_datapacks(&config));
        }
    }

    #[test]
    fn test_logs() {
        let mut subcommands = get_subcommands();
        let config = get_config();

        subcommands
            .container_provider
            .expect_display_container_logs()
            .with(eq(config.clone()))
            .times(1)
            .returning(|_| Ok(()));

        assert_eq!(Ok(()), subcommands.logs(&config));
    }
}
