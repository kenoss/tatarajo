use crate::util::Id;
use crate::view::stackset::StackSet;
use crate::view::window::{Window, WindowProps};
use smithay::utils::{Logical, Rectangle};

pub struct ViewLayoutApi<'state> {
    pub(super) stackset: &'state StackSet,
    pub(super) rect: Rectangle<i32, Logical>,
    pub(super) layout_queue: Vec<(Id<Window>, WindowProps)>,
}

impl ViewLayoutApi<'_> {
    pub fn stackset(&self) -> &StackSet {
        self.stackset
    }

    pub fn rect(&self) -> &Rectangle<i32, Logical> {
        &self.rect
    }

    pub fn layout_window(&mut self, id: Id<Window>, geometry: Rectangle<i32, Logical>) {
        // TODO: Check that id is not already registered.
        let props = WindowProps { geometry };
        self.layout_queue.push((id, props));
    }
}
