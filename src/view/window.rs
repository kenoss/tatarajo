use crate::util::Id;

#[derive(Debug, Clone)]
pub struct Window {
    id: Id<Self>,
}

impl Window {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { id: Id::new() }
    }

    pub fn id(&self) -> Id<Self> {
        self.id
    }
}
