use crate::Sabiniwm;

#[derive(Debug, Clone)]
pub enum Command {
    Spawn(String),
}

impl Command {
    pub fn spawn(s: impl ToString) -> Self {
        Command::Spawn(s.to_string())
    }
}

impl Sabiniwm {
    pub(crate) fn process_command(&mut self, command: &Command) {
        match command {
            Command::Spawn(s) => {
                let _ = std::process::Command::new("/bin/sh")
                    .arg("-c")
                    .arg(s)
                    .spawn();
            }
        }
    }
}
