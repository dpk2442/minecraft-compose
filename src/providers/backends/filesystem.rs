use std::fs;
use std::path;

#[cfg_attr(test, mockall::automock)]
pub trait FilesystemBackend {
    fn canonicalize_path(&self, path: &path::PathBuf) -> Result<path::PathBuf, ()>;
    fn directory_exists(&self, directory_path: &path::PathBuf) -> bool;
    fn create_directory(&self, directory_path: &path::PathBuf) -> Result<(), ()>;
    fn file_exists(&self, file_path: &path::PathBuf) -> bool;
    fn read_file(&self, file_path: &path::PathBuf) -> Result<String, ()>;
    fn write_file(&self, file_path: &path::PathBuf, contents: &str) -> Result<(), ()>;
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
        fs::create_dir(directory_path).or_else(|err| {
            log::trace!(
                "Unable to create folder {}: {}",
                directory_path.display(),
                err
            );
            Err(())
        })
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
}

pub fn new_from_defaults() -> FilesystemBackendImpl {
    FilesystemBackendImpl {}
}
