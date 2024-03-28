use crate::util::Id;
use crate::util::{FocusedVec, NonEmptyFocusedVec};
use crate::view::layout_node::LayoutNode;
use crate::view::window::Window;

pub struct StackSet {
    pub(super) workspaces: NonEmptyFocusedVec<Workspace>,
}

pub struct Workspace {
    // tag: String,
    // FIXME: LayoutNode -> StackLayout
    pub(super) layouts: NonEmptyFocusedVec<Id<LayoutNode>>,
    pub(super) stack: FocusedVec<Id<Window>>,
}

impl StackSet {
    pub(super) fn new(layouts: Vec<Id<LayoutNode>>) -> Self {
        let workspace = Workspace {
            layouts: NonEmptyFocusedVec::new(layouts, 0),
            stack: FocusedVec::default(),
        };
        Self {
            workspaces: NonEmptyFocusedVec::new(vec![workspace], 0),
        }
    }

    pub fn workspaces(&self) -> &NonEmptyFocusedVec<Workspace> {
        &self.workspaces
    }
}

impl Workspace {
    pub fn stack(&self) -> &FocusedVec<Id<Window>> {
        &self.stack
    }

    pub fn layouts(&self) -> &NonEmptyFocusedVec<Id<LayoutNode>> {
        &self.layouts
    }

    pub fn focus_next_window(&mut self, count: isize) {
        if !self.stack.is_empty() {
            let n = self.stack.len() as isize;
            let i = self.stack.focused_index() as isize;
            let i = i + count;
            let i = ((i % n) + n) % n; // modulo n
            *self.stack.focused_index_mut() = i as usize;
        }
    }
}
