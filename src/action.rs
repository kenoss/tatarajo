use crate::Sabiniwm;

#[derive(Debug, Clone)]
pub enum Action {
    Spawn(String),
}

impl Action {
    pub fn spawn(s: impl ToString) -> Self {
        Action::Spawn(s.to_string())
    }
}

impl Sabiniwm {
    pub(crate) fn process_action(&mut self, action: &Action) {
        match action {
            Action::Spawn(s) => {
                let _ = std::process::Command::new("/bin/sh")
                    .arg("-c")
                    .arg(s)
                    .spawn();
            }
        }
    }
}
