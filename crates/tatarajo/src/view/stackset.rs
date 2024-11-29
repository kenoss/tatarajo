use crate::util::{FocusedVec, Id, NonEmptyFocusedVec};
use crate::view::window::Window;

pub struct StackSet {
    pub workspaces: NonEmptyFocusedVec<Workspace>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceTag(pub String);

pub struct Workspace {
    pub tag: WorkspaceTag,
    pub stack: FocusedVec<Id<Window>>,
}

impl StackSet {
    pub(super) fn new(tags: Vec<WorkspaceTag>) -> Self {
        let workspaces = tags
            .into_iter()
            .map(|tag| Workspace {
                tag,
                stack: FocusedVec::default(),
            })
            .collect();
        let workspaces = NonEmptyFocusedVec::new(workspaces, 0);
        Self { workspaces }
    }

    pub fn workspaces(&self) -> &NonEmptyFocusedVec<Workspace> {
        &self.workspaces
    }
}

impl Workspace {
    pub fn stack(&self) -> &FocusedVec<Id<Window>> {
        &self.stack
    }
}
