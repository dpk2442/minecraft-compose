use std::collections::{HashMap, HashSet};
use std::path;

use crate::providers::backends::filesystem::{self, FilesystemBackend, FilesystemBackendImpl};

#[cfg_attr(test, mockall::automock)]
pub trait FileProvider {
    fn get_data_path(&self) -> Result<path::PathBuf, ()>;
    fn create_data_folder(&self) -> Result<(), ()>;
    fn create_and_populate_server_properties(&self) -> Result<(), ()>;
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

    fn create_and_populate_server_properties(&self) -> Result<(), ()> {
        let server_properties = match self
            .filesystem_backend
            .file_exists(&self.server_properties_path)
        {
            true => self
                .filesystem_backend
                .read_file(&self.server_properties_path)?,
            false => "".to_owned(),
        };

        let mut keys = self
            .default_properties
            .keys()
            .clone()
            .collect::<HashSet<&String>>();
        let mut new_properties = server_properties
            .lines()
            .map(|line| match line.find("=") {
                None => line.to_owned(),
                Some(i) => {
                    let line_key = String::from(&line[0..i]);
                    match keys.remove(&line_key) {
                        false => line.to_owned(),
                        true => format!("{}={}", line_key, self.default_properties[&line_key]),
                    }
                }
            })
            .collect::<Vec<String>>();

        for key in keys.iter() {
            new_properties.push(format!("{}={}", key, self.default_properties[*key]));
        }

        self.filesystem_backend
            .write_file(&self.server_properties_path, &new_properties.join("\n"))?;

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
    use crate::providers::backends::filesystem::MockFilesystemBackend;

    fn get_file_provider() -> FileProviderImpl<MockFilesystemBackend> {
        FileProviderImpl::new(MockFilesystemBackend::new())
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
                file_provider.create_and_populate_server_properties()
            );
        }

        #[test]
        fn empty_file_exists() {
            let mut file_provider = get_file_provider();

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
                file_provider.create_and_populate_server_properties()
            );
        }

        #[test]
        fn non_empty_file_exists() {
            let mut file_provider = get_file_provider();

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
                file_provider.create_and_populate_server_properties()
            );
        }
    }
}
