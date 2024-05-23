use crate::util::Id;
use crate::view::layout_node::{LayoutMessage, LayoutNode};
use crate::view::stackset::StackSet;
use crate::view::view::ViewState;
use crate::view::window::{Border, Rgba, Window, WindowProps};
use smithay::utils::{Logical, Rectangle};

pub struct ViewLayoutApi<'state> {
    pub(super) state: &'state mut ViewState,
    pub(super) rect: Rectangle<i32, Logical>,
}

impl ViewLayoutApi<'_> {
    pub fn stackset(&self) -> &StackSet {
        &self.state.stackset
    }

    pub fn rect(&self) -> &Rectangle<i32, Logical> {
        &self.rect
    }

    pub fn layout_node(&mut self, id: Id<LayoutNode>, rect: Rectangle<i32, Logical>) {
        assert!(self.rect.contains_rect(rect));

        // Safety: LayoutNode is borrowed only by this method and this method doesn't allow recursive use of LayoutNode.
        // TODO: Consider to use Rc and Weak.
        let node = self.state.nodes.get(&id).unwrap().as_ptr();
        let node = unsafe { &*node };
        let mut api = ViewLayoutApi {
            state: self.state,
            rect,
        };
        node.layout(&mut api);
    }

    pub fn layout_window(&mut self, id: Id<Window>, geometry: Rectangle<i32, Logical>) {
        // TODO: Check that id is not already registered.
        let border = Border {
            dim: 0.into(),
            active_rgba: Rgba::from_rgba(0x000000ff),
            inactive_rgba: Rgba::from_rgba(0x000000ff),
        };
        let props = WindowProps { geometry, border };
        self.state.layout_queue.push((id, props));
    }

    pub fn modify_layout_queue_with<F>(&mut self, f: F)
    where
        F: Fn(&mut Vec<(Id<Window>, WindowProps)>),
    {
        f(&mut self.state.layout_queue);
    }
}

pub struct ViewHandleMessageApi<'state> {
    pub(super) state: &'state mut ViewState,
}

impl ViewHandleMessageApi<'_> {
    pub fn stackset(&self) -> &StackSet {
        &self.state.stackset
    }

    pub fn handle_message(
        &mut self,
        id: Id<LayoutNode>,
        message: &LayoutMessage,
    ) -> std::ops::ControlFlow<()> {
        // Safety: LayoutNode is borrowed only by this method and this method doesn't allow recursive use of LayoutNode.
        // TODO: Consider to use Rc and Weak.
        let node = self.state.nodes.get_mut(&id).unwrap().as_ptr();
        let node = unsafe { &mut *node };
        let mut api = ViewHandleMessageApi { state: self.state };
        node.handle_message(&mut api, message)
    }
}
