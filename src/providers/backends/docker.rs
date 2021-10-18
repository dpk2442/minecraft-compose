use bollard::container::{Config, CreateContainerOptions, LogsOptions};
use bollard::errors;
use bollard::models::ContainerInspectResponse;

#[derive(Debug)]
pub enum InspectResult {
    Ok(ContainerInspectResponse),
    NotFound,
}

#[cfg_attr(test, mockall::automock)]
pub trait DockerBackend {
    fn download_image(&self, image: &str, tag: &str) -> Result<(), ()>;
    fn create_container(&self, name: &str, container_config: Config<String>) -> Result<(), ()>;
    fn delete_container(&self, name: &str) -> Result<(), ()>;
    fn start_container(&self, name: &str) -> Result<(), ()>;
    fn stop_container(&self, name: &str) -> Result<(), ()>;
    fn inspect_container(&self, name: &str) -> Result<InspectResult, ()>;
    fn get_container_logs(&self, name: &str) -> Box<dyn Iterator<Item = Result<String, ()>>>;
}

pub struct DockerBackendImpl {
    docker: bollard::Docker,
}

pub fn new_from_defaults() -> Result<DockerBackendImpl, ()> {
    Ok(DockerBackendImpl {
        docker: bollard::Docker::connect_with_local_defaults().or_else(|err| {
            log::debug!("Failed to connect to docker: {}", err);
            Err(())
        })?,
    })
}

impl DockerBackend for DockerBackendImpl {
    fn download_image(&self, image: &str, tag: &str) -> Result<(), ()> {
        log::trace!("Downloading image {}:{}", image, tag);

        for item in futures::executor::block_on_stream(self.docker.create_image(
            Some(bollard::image::CreateImageOptions {
                from_image: image.to_owned(),
                tag: tag.to_owned(),
                ..std::default::Default::default()
            }),
            None,
            None,
        )) {
            match item {
                Ok(_) => Ok(()),
                Err(err) => {
                    log::trace!("Unable to download image {}:{}: {}", image, tag, err);
                    Err(())
                }
            }?;
        }

        Ok(())
    }

    fn create_container(&self, name: &str, container_config: Config<String>) -> Result<(), ()> {
        log::trace!("Creating container {}", name);
        match futures::executor::block_on(self.docker.create_container(
            Some(CreateContainerOptions { name: name }),
            container_config,
        )) {
            Ok(response) => {
                response.warnings.iter().for_each(|warning| {
                    log::warn!("Warning: {}", warning);
                });
                Ok(())
            }
            Err(err) => {
                log::trace!("Unable to create container {}: {}", name, err);
                Err(())
            }
        }
    }

    fn delete_container(&self, name: &str) -> Result<(), ()> {
        log::trace!("Deleting container {}", name);
        futures::executor::block_on(self.docker.remove_container(name, None)).or_else(|err| {
            log::trace!("Unable to delete container {}: {}", name, err);
            Err(())
        })
    }

    fn start_container(&self, name: &str) -> Result<(), ()> {
        log::trace!("Starting container {}", name);
        futures::executor::block_on(self.docker.start_container::<String>(name, None)).or_else(
            |err| {
                log::trace!("Unable to start container {}: {}", name, err);
                Err(())
            },
        )
    }

    fn stop_container(&self, name: &str) -> Result<(), ()> {
        log::trace!("Stopping container {}", name);
        futures::executor::block_on(self.docker.stop_container(name, None)).or_else(|err| {
            log::trace!("Unable to start container {}: {}", name, err);
            Err(())
        })
    }

    fn inspect_container(&self, name: &str) -> Result<InspectResult, ()> {
        log::trace!("Inspecting container {}", name);
        match futures::executor::block_on(self.docker.inspect_container(name, None)) {
            Ok(result) => Ok(InspectResult::Ok(result)),
            Err(errors::Error::DockerResponseNotFoundError { message: _ }) => {
                Ok(InspectResult::NotFound)
            }
            Err(err) => {
                log::trace!("Unable to inspect container {}: {}", name, err);
                Err(())
            }
        }
    }

    fn get_container_logs(&self, name: &str) -> Box<dyn Iterator<Item = Result<String, ()>>> {
        log::trace!("Getting logs for container {}", name);

        Box::new(
            futures::executor::block_on_stream(self.docker.logs(
                &name,
                Some(LogsOptions {
                    follow: true,
                    stdout: true,
                    stderr: true,
                    tail: "10",
                    ..std::default::Default::default()
                }),
            ))
            .map(|line_result| match line_result {
                Ok(output) => Ok(String::from_utf8_lossy(&output.into_bytes()).to_string()),
                Err(err) => {
                    log::trace!("Error reading logs: {}", err);
                    Err(())
                }
            }),
        )
    }
}
