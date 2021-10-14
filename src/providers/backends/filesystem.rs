use std::fs;
use std::path;

#[cfg_attr(test, mockall::automock)]
pub trait FilesystemBackend {
    fn canonicalize_path(&self, path: &path::PathBuf) -> Result<path::PathBuf, ()>;
    fn directory_exists(&self, directory_path: &path::PathBuf) -> bool;
    fn create_directory(&self, directory_path: &path::PathBuf) -> Result<(), ()>;
    fn read_directory(&self, directory_path: &path::PathBuf) -> Result<Vec<path::PathBuf>, ()>;
    fn file_exists(&self, file_path: &path::PathBuf) -> bool;
    fn read_file(&self, file_path: &path::PathBuf) -> Result<String, ()>;
    fn write_file(&self, file_path: &path::PathBuf, contents: &str) -> Result<(), ()>;
    fn copy_file(&self, src: &path::PathBuf, dest: &path::PathBuf) -> Result<(), ()>;
    fn delete_file(&self, file_path: &path::PathBuf) -> Result<(), ()>;
}

pub struct FilesystemBackendImpl {}

impl FilesystemBackend for FilesystemBackendImpl {
    fn canonicalize_path(&self, path: &path::PathBuf) -> Result<path::PathBuf, ()> {
        std::fs::canonicalize(path).or_else(|err| {
            log::trace!("Unable to canonicalize path {}: {}", path.display(), err);
            Err(())
        })
    }

    fn directory_exists(&self, directory_path: &path::PathBuf) -> bool {
        directory_path.is_dir()
    }

    fn create_directory(&self, directory_path: &path::PathBuf) -> Result<(), ()> {
        fs::create_dir_all(directory_path).or_else(|err| {
            log::trace!(
                "Unable to create folder {}: {}",
                directory_path.display(),
                err
            );
            Err(())
        })
    }

    fn read_directory(&self, directory_path: &path::PathBuf) -> Result<Vec<path::PathBuf>, ()> {
        match fs::read_dir(directory_path) {
            Ok(read_dir) => {
                let mut entries = vec![];
                for dir_entry_result in read_dir {
                    match dir_entry_result {
                        Ok(dir_entry) => {
                            entries.push(dir_entry.path());
                        }
                        Err(err) => {
                            log::trace!(
                                "Unable to read directory {}: {}",
                                directory_path.display(),
                                err
                            );
                            return Err(());
                        }
                    }
                }

                Ok(entries)
            }
            Err(err) => {
                log::trace!(
                    "Unable to read directory {}: {}",
                    directory_path.display(),
                    err
                );
                Err(())
            }
        }
    }

    fn file_exists(&self, file_path: &path::PathBuf) -> bool {
        file_path.is_file()
    }

    fn read_file(&self, file_path: &path::PathBuf) -> Result<String, ()> {
        fs::read_to_string(file_path).or_else(|err| {
            log::trace!("Unable to read file {}: {}", file_path.display(), err);
            Err(())
        })
    }

    fn write_file(&self, file_path: &path::PathBuf, contents: &str) -> Result<(), ()> {
        fs::write(file_path, contents).or_else(|err| {
            log::trace!("Unable to write file {}: {}", file_path.display(), err);
            Err(())
        })
    }

    fn copy_file(&self, src: &path::PathBuf, dest: &path::PathBuf) -> Result<(), ()> {
        match fs::copy(src, dest) {
            Ok(_) => Ok(()),
            Err(err) => {
                log::trace!(
                    "Unable to copy file \"{}\" to \"{}\": {}",
                    src.display(),
                    dest.display(),
                    err
                );
                Err(())
            }
        }
    }

    fn delete_file(&self, file_path: &path::PathBuf) -> Result<(), ()> {
        fs::remove_file(file_path).or_else(|err| {
            log::trace!("Unable to delete file \"{}\": {}", file_path.display(), err);
            Err(())
        })
    }
}

pub fn new_from_defaults() -> FilesystemBackendImpl {
    FilesystemBackendImpl {}
}
