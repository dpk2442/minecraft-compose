use crate::config::Config;
use std::collections::{HashMap, HashSet};
use std::path;

use crate::providers::backends::filesystem::{self, FilesystemBackend, FilesystemBackendImpl};

#[cfg_attr(test, mockall::automock)]
pub trait FileProvider {
    fn get_data_path(&self) -> Result<path::PathBuf, ()>;
    fn create_data_folder(&self) -> Result<(), ()>;
    fn create_and_populate_server_properties(&self, config: &Config) -> Result<(), ()>;
    fn sync_datapacks(&self, config: &Config) -> Result<(), ()>;
}

pub struct FileProviderImpl<T: FilesystemBackend> {
    data_path: path::PathBuf,
    server_properties_path: path::PathBuf,
    default_properties: HashMap<String, String>,
    filesystem_backend: T,
}

impl<T: FilesystemBackend> FileProviderImpl<T> {
    fn new(filesystem_backend: T) -> FileProviderImpl<T> {
        let data_path = path::Path::new("data").to_owned();
        let server_properties_path = data_path.join("server.properties");

        let mut default_properties = HashMap::new();
        default_properties.insert("server-port".to_owned(), "25565".to_owned());
        default_properties.insert("enable-rcon".to_owned(), "true".to_owned());
        default_properties.insert("rcon.port".to_owned(), "25575".to_owned());
        default_properties.insert("rcon.password".to_owned(), "minecraft".to_owned());
        default_properties.insert("broadcast-rcon-to-ops".to_owned(), "true".to_owned());

        FileProviderImpl {
            data_path,
            server_properties_path,
            default_properties,
            filesystem_backend: filesystem_backend,
        }
    }
}

impl<T: FilesystemBackend> FileProvider for FileProviderImpl<T> {
    fn get_data_path(&self) -> Result<path::PathBuf, ()> {
        self.filesystem_backend.canonicalize_path(&self.data_path)
    }

    fn create_data_folder(&self) -> Result<(), ()> {
        if !self.filesystem_backend.directory_exists(&self.data_path) {
            self.filesystem_backend.create_directory(&self.data_path)?;
        }

        Ok(())
    }

    fn create_and_populate_server_properties(&self, config: &Config) -> Result<(), ()> {
        let server_properties = match self
            .filesystem_backend
            .file_exists(&self.server_properties_path)
        {
            true => self
                .filesystem_backend
                .read_file(&self.server_properties_path)?,
            false => "".to_owned(),
        };

        let mut properties_to_remove = HashSet::new();
        let mut properties_to_set = self.default_properties.clone();

        properties_to_set.insert("level-name".to_owned(), config.world.name.clone());
        properties_to_set.insert("gamemode".to_owned(), config.world.gamemode.clone());
        properties_to_set.insert("difficulty".to_owned(), config.world.difficulty.clone());
        properties_to_set.insert(
            "allow-flight".to_owned(),
            config.world.allow_flight.to_string(),
        );
        match &config.world.seed {
            Some(seed) => drop(properties_to_set.insert("level-seed".to_owned(), seed.clone())),
            None => drop(properties_to_remove.insert("level-seed".to_owned())),
        };

        let mut keys = properties_to_set
            .keys()
            .clone()
            .collect::<HashSet<&String>>();
        let mut new_properties = server_properties
            .lines()
            .filter_map(|line| match line.find("=") {
                None => Some(line.to_owned()),
                Some(i) => {
                    let line_key = String::from(&line[0..i]);
                    match keys.remove(&line_key) {
                        false => match properties_to_remove.contains(&line_key) {
                            true => None,
                            false => Some(line.to_owned()),
                        },
                        true => Some(format!("{}={}", line_key, properties_to_set[&line_key])),
                    }
                }
            })
            .collect::<Vec<String>>();

        for key in keys.iter() {
            new_properties.push(format!("{}={}", key, properties_to_set[*key]));
        }

        self.filesystem_backend
            .write_file(&self.server_properties_path, &new_properties.join("\n"))?;

        Ok(())
    }

    fn sync_datapacks(&self, config: &Config) -> Result<(), ()> {
        let installed_datapacks_path = self.data_path.join(&config.world.name).join("datapacks");
        if !self
            .filesystem_backend
            .directory_exists(&installed_datapacks_path)
        {
            self.filesystem_backend
                .create_directory(&installed_datapacks_path)?;
        }

        let installed_datapacks_path = self
            .filesystem_backend
            .canonicalize_path(&installed_datapacks_path)?;

        let datapacks_to_install = match &config.datapacks {
            Some(datapacks) => datapacks.clone(),
            None => HashMap::new(),
        };

        for datapack_path in self
            .filesystem_backend
            .read_directory(&installed_datapacks_path)?
            .iter()
            .filter(|entry| match entry.file_stem() {
                None => false,
                Some(stem) => {
                    !datapacks_to_install.contains_key(&stem.to_string_lossy().to_string())
                }
            })
        {
            log::trace!("Uninstalling datapack \"{}\"", datapack_path.display());
            self.filesystem_backend.delete_file(&datapack_path)?;
        }

        for (datapack_name, datapack_src) in datapacks_to_install.iter() {
            let datapack_src_path = match self
                .filesystem_backend
                .canonicalize_path(&path::Path::new("datapacks").join(datapack_src))
            {
                Ok(path) => path,
                Err(()) => {
                    log::warn!(
                        "Unable to find the source for the datapack \"{}\", skipping",
                        datapack_name
                    );
                    continue;
                }
            };

            let datapack_dest_path =
                installed_datapacks_path.join(&format!("{}.zip", datapack_name));
            log::trace!(
                "Installing \"{}\" from \"{}\" to \"{}\"",
                datapack_name,
                datapack_src_path.display(),
                datapack_dest_path.display(),
            );

            self.filesystem_backend
                .copy_file(&datapack_src_path, &datapack_dest_path)?;
        }

        Ok(())
    }
}

pub fn new_from_defaults() -> FileProviderImpl<FilesystemBackendImpl> {
    FileProviderImpl::new(filesystem::new_from_defaults())
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use super::*;
    use crate::config;
    use crate::providers::backends::filesystem::MockFilesystemBackend;

    fn get_file_provider() -> FileProviderImpl<MockFilesystemBackend> {
        FileProviderImpl::new(MockFilesystemBackend::new())
    }

    fn get_config() -> Config {
        Config {
            name: "name".to_owned(),
            host: "0.0.0.0".to_owned(),
            port: 25565,
            server: config::Server {
                version: "1.17.1".to_owned(),
                server_type: config::ServerType::Vanilla,
                ..std::default::Default::default()
            },
            world: config::World {
                name: "world".to_owned(),
                gamemode: "survival".to_owned(),
                difficulty: "easy".to_owned(),
                allow_flight: false,
                ..std::default::Default::default()
            },
            ..std::default::Default::default()
        }
    }

    fn compare_server_properties(expected: &str, actual: &str) {
        let mut expected_lines = expected.lines().collect::<Vec<&str>>();
        expected_lines.sort();

        let mut actual_lines = actual.lines().collect::<Vec<&str>>();
        actual_lines.sort();

        assert_eq!(expected_lines, actual_lines)
    }

    mod test_create_data_folder {
        use super::*;

        #[test]
        fn does_not_exist() {
            let mut file_provider = get_file_provider();

            file_provider
                .filesystem_backend
                .expect_directory_exists()
                .with(eq(path::Path::new("data").to_path_buf()))
                .times(1)
                .returning(|_| false);

            file_provider
                .filesystem_backend
                .expect_create_directory()
                .with(eq(path::Path::new("data").to_path_buf()))
                .times(1)
                .returning(|_| Ok(()));

            assert_eq!(Ok(()), file_provider.create_data_folder());
        }

        #[test]
        fn exists() {
            let mut file_provider = get_file_provider();

            file_provider
                .filesystem_backend
                .expect_directory_exists()
                .with(eq(path::Path::new("data").to_path_buf()))
                .times(1)
                .returning(|_| true);

            file_provider
                .filesystem_backend
                .expect_create_directory()
                .times(0);

            assert_eq!(Ok(()), file_provider.create_data_folder());
        }
    }

    mod test_create_and_populate_server_properties {
        use super::*;

        #[test]
        fn file_not_exist() {
            let mut file_provider = get_file_provider();
            let config = get_config();

            file_provider
                .filesystem_backend
                .expect_file_exists()
                .with(eq(path::Path::new("data").join("server.properties")))
                .times(1)
                .returning(|_| false);

            file_provider
                .filesystem_backend
                .expect_write_file()
                .with(
                    eq(path::Path::new("data").join("server.properties")),
                    mockall::predicate::function(|actual_props: &str| {
                        compare_server_properties(
                            concat!(
                                "server-port=25565\n",
                                "enable-rcon=true\n",
                                "rcon.port=25575\n",
                                "rcon.password=minecraft\n",
                                "broadcast-rcon-to-ops=true\n",
                                "level-name=world\n",
                                "gamemode=survival\n",
                                "difficulty=easy\n",
                                "allow-flight=false\n",
                            ),
                            actual_props,
                        );
                        true
                    }),
                )
                .times(1)
                .returning(|_, _| Ok(()));

            assert_eq!(
                Ok(()),
                file_provider.create_and_populate_server_properties(&config)
            );
        }

        #[test]
        fn empty_file_exists() {
            let mut file_provider = get_file_provider();
            let config = get_config();

            file_provider
                .filesystem_backend
                .expect_file_exists()
                .with(eq(path::Path::new("data").join("server.properties")))
                .times(1)
                .returning(|_| true);

            file_provider
                .filesystem_backend
                .expect_read_file()
                .times(1)
                .returning(|_| Ok(String::from("")));

            file_provider
                .filesystem_backend
                .expect_write_file()
                .with(
                    eq(path::Path::new("data").join("server.properties")),
                    mockall::predicate::function(|actual_props: &str| {
                        compare_server_properties(
                            concat!(
                                "server-port=25565\n",
                                "enable-rcon=true\n",
                                "rcon.port=25575\n",
                                "rcon.password=minecraft\n",
                                "broadcast-rcon-to-ops=true\n",
                                "level-name=world\n",
                                "gamemode=survival\n",
                                "difficulty=easy\n",
                                "allow-flight=false\n",
                            ),
                            actual_props,
                        );
                        true
                    }),
                )
                .times(1)
                .returning(|_, _| Ok(()));

            assert_eq!(
                Ok(()),
                file_provider.create_and_populate_server_properties(&config)
            );
        }

        #[test]
        fn non_empty_file_exists() {
            let mut file_provider = get_file_provider();
            let config = get_config();

            file_provider
                .filesystem_backend
                .expect_file_exists()
                .with(eq(path::Path::new("data").join("server.properties")))
                .times(1)
                .returning(|_| true);

            file_provider
                .filesystem_backend
                .expect_read_file()
                .times(1)
                .returning(|_| {
                    Ok(String::from(concat!(
                        "rcon.port=25575\n",
                        "server-port=25566\n",
                        "rcon.password=minecraft\n",
                    )))
                });

            file_provider
                .filesystem_backend
                .expect_write_file()
                .with(
                    eq(path::Path::new("data").join("server.properties")),
                    mockall::predicate::function(|actual_props: &str| {
                        compare_server_properties(
                            concat!(
                                "server-port=25565\n",
                                "enable-rcon=true\n",
                                "rcon.port=25575\n",
                                "rcon.password=minecraft\n",
                                "broadcast-rcon-to-ops=true\n",
                                "level-name=world\n",
                                "gamemode=survival\n",
                                "difficulty=easy\n",
                                "allow-flight=false\n",
                            ),
                            actual_props,
                        );
                        true
                    }),
                )
                .times(1)
                .returning(|_, _| Ok(()));

            assert_eq!(
                Ok(()),
                file_provider.create_and_populate_server_properties(&config)
            );
        }

        #[test]
        fn test_removes_seed() {
            let mut file_provider = get_file_provider();
            let config = get_config();

            file_provider
                .filesystem_backend
                .expect_file_exists()
                .with(eq(path::Path::new("data").join("server.properties")))
                .times(1)
                .returning(|_| true);

            file_provider
                .filesystem_backend
                .expect_read_file()
                .times(1)
                .returning(|_| Ok(String::from("level-seed=test-seed\n")));

            file_provider
                .filesystem_backend
                .expect_write_file()
                .with(
                    eq(path::Path::new("data").join("server.properties")),
                    mockall::predicate::function(|actual_props: &str| {
                        compare_server_properties(
                            concat!(
                                "server-port=25565\n",
                                "enable-rcon=true\n",
                                "rcon.port=25575\n",
                                "rcon.password=minecraft\n",
                                "broadcast-rcon-to-ops=true\n",
                                "level-name=world\n",
                                "gamemode=survival\n",
                                "difficulty=easy\n",
                                "allow-flight=false\n",
                            ),
                            actual_props,
                        );
                        true
                    }),
                )
                .times(1)
                .returning(|_, _| Ok(()));

            assert_eq!(
                Ok(()),
                file_provider.create_and_populate_server_properties(&config)
            );
        }
    }

    mod test_sync_datapacks {
        use super::*;

        fn get_datapacks_config() -> Config {
            let mut datapacks = HashMap::new();
            datapacks.insert("datapack1".to_owned(), "source_path_1".to_owned());
            datapacks.insert("datapack2".to_owned(), "source_path_2".to_owned());

            let mut config = get_config();
            config.datapacks = Some(datapacks);
            config
        }

        fn expect_canonicalize_paths(
            filesystem_backend: &mut MockFilesystemBackend,
            paths: Vec<path::PathBuf>,
        ) {
            for path in paths.iter() {
                filesystem_backend
                    .expect_canonicalize_path()
                    .with(eq(path.clone()))
                    .times(1)
                    .returning(|p| Ok(p.clone()));
            }
        }

        fn expect_copy_files(
            filesystem_backend: &mut MockFilesystemBackend,
            paths: Vec<(path::PathBuf, path::PathBuf)>,
        ) {
            for (src, dest) in paths.iter() {
                filesystem_backend
                    .expect_copy_file()
                    .with(eq(src.clone()), eq(dest.clone()))
                    .times(1)
                    .returning(|_, _| Ok(()));
            }
        }

        fn installs_datapacks(
            world_datapacks_path: path::PathBuf,
            config: Config,
            mut file_provider: FileProviderImpl<MockFilesystemBackend>,
        ) {
            expect_canonicalize_paths(
                &mut file_provider.filesystem_backend,
                vec![
                    path::Path::new("data").join("world").join("datapacks"),
                    path::Path::new("datapacks").join("source_path_1"),
                    path::Path::new("datapacks").join("source_path_2"),
                ],
            );

            expect_copy_files(
                &mut file_provider.filesystem_backend,
                vec![
                    (
                        path::Path::new("datapacks").join("source_path_1"),
                        world_datapacks_path.join("datapack1.zip"),
                    ),
                    (
                        path::Path::new("datapacks").join("source_path_2"),
                        world_datapacks_path.join("datapack2.zip"),
                    ),
                ],
            );

            assert_eq!(Ok(()), file_provider.sync_datapacks(&config));
        }

        #[test]
        fn no_installed_datapacks_directory_exists() {
            let world_datapacks_path = path::Path::new("data").join("world").join("datapacks");
            let config = get_datapacks_config();
            let mut file_provider = get_file_provider();

            file_provider
                .filesystem_backend
                .expect_directory_exists()
                .with(eq(world_datapacks_path.clone()))
                .times(1)
                .returning(|_| true);

            file_provider
                .filesystem_backend
                .expect_read_directory()
                .with(eq(world_datapacks_path.clone()))
                .times(1)
                .returning(|_| Ok(vec![]));

            installs_datapacks(world_datapacks_path, config, file_provider);
        }

        #[test]
        fn no_installed_datapacks_directory_does_not_exist() {
            let world_datapacks_path = path::Path::new("data").join("world").join("datapacks");
            let config = get_datapacks_config();
            let mut file_provider = get_file_provider();

            file_provider
                .filesystem_backend
                .expect_directory_exists()
                .with(eq(world_datapacks_path.clone()))
                .times(1)
                .returning(|_| false);

            file_provider
                .filesystem_backend
                .expect_create_directory()
                .with(eq(world_datapacks_path.clone()))
                .times(1)
                .returning(|_| Ok(()));

            file_provider
                .filesystem_backend
                .expect_read_directory()
                .with(eq(world_datapacks_path.clone()))
                .times(1)
                .returning(|_| Ok(vec![]));

            installs_datapacks(world_datapacks_path, config, file_provider);
        }

        #[test]
        fn expected_datapack_installed() {
            let world_datapacks_path = path::Path::new("data").join("world").join("datapacks");
            let config = get_datapacks_config();
            let mut file_provider = get_file_provider();

            file_provider
                .filesystem_backend
                .expect_directory_exists()
                .with(eq(world_datapacks_path.clone()))
                .times(1)
                .returning(|_| true);

            let world_datapacks_path_clone = world_datapacks_path.clone();
            file_provider
                .filesystem_backend
                .expect_read_directory()
                .with(eq(world_datapacks_path.clone()))
                .times(1)
                .returning(move |_| Ok(vec![world_datapacks_path_clone.join("datapack1.zip")]));

            installs_datapacks(world_datapacks_path, config, file_provider);
        }

        #[test]
        fn unexpected_datapack_installed() {
            let world_datapacks_path = path::Path::new("data").join("world").join("datapacks");
            let config = get_datapacks_config();
            let mut file_provider = get_file_provider();

            file_provider
                .filesystem_backend
                .expect_directory_exists()
                .with(eq(world_datapacks_path.clone()))
                .times(1)
                .returning(|_| true);

            let world_datapacks_path_clone = world_datapacks_path.clone();
            file_provider
                .filesystem_backend
                .expect_read_directory()
                .with(eq(world_datapacks_path.clone()))
                .times(1)
                .returning(move |_| Ok(vec![world_datapacks_path_clone.join("datapack3.zip")]));

            file_provider
                .filesystem_backend
                .expect_delete_file()
                .with(eq(world_datapacks_path.join("datapack3.zip")))
                .times(1)
                .returning(|_| Ok(()));

            installs_datapacks(world_datapacks_path, config, file_provider);
        }
    }
}
