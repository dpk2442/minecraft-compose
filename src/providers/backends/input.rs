use rustyline::{config::Config, error::ReadlineError, Editor};

#[cfg_attr(test, mockall::automock(type Output = MockInputBackend;))]
pub trait InputBackendFactory {
    type Output: InputBackend;

    fn create(&self) -> Self::Output;
}

pub struct InputBackendFactoryImpl {}

impl InputBackendFactory for InputBackendFactoryImpl {
    type Output = InputBackendImpl;

    fn create(&self) -> InputBackendImpl {
        InputBackendImpl {
            editor: Editor::with_config(Config::builder().auto_add_history(true).build()),
        }
    }
}

#[derive(Clone)]
pub enum InputResponse {
    Input(String),
    EndOfInput,
}

#[cfg_attr(test, mockall::automock)]
pub trait InputBackend {
    fn get_line(&mut self, prompt: &str) -> Result<InputResponse, ()>;
}

pub struct InputBackendImpl {
    editor: Editor<()>,
}

impl InputBackend for InputBackendImpl {
    fn get_line(&mut self, prompt: &str) -> Result<InputResponse, ()> {
        match self.editor.readline(prompt) {
            Ok(line) => Ok(InputResponse::Input(line)),
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                Ok(InputResponse::EndOfInput)
            }
            Err(err) => {
                log::trace!("Encountered a readline error: {}", err);
                Err(())
            }
        }
    }
}
