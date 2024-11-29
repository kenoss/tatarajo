use crate::model::grid_geometry::{RectangleExt, SplitSpec};
use crate::util::{Id, NonEmptyFocusedVec};
use crate::view::api::{ViewHandleMessageApi, ViewLayoutApi};
use crate::view::layout_node::{LayoutMessage, LayoutMessageI, LayoutNode, LayoutNodeI};
pub use itertools::izip;

pub struct LayoutFull {}

impl LayoutNodeI for LayoutFull {
    fn layout(&self, api: &mut ViewLayoutApi<'_>) {
        if let Some(&window_id) = api.stackset().workspaces().focus().stack().focus() {
            api.layout_window(window_id, *api.rect());
        }
    }
}

pub struct LayoutTall {}

impl LayoutNodeI for LayoutTall {
    fn layout(&self, api: &mut ViewLayoutApi<'_>) {
        let mut head = api.stackset().workspaces().focus().stack().as_vec().clone();
        match head.len() {
            0 => {}
            1 => {
                api.layout_window(head[0], *api.rect());
            }
            _ => {
                let tail = head.split_off(1);
                let [head_rect, tail_rect] = api
                    .rect()
                    .split_vertically_2([SplitSpec::Elastic, SplitSpec::Elastic]);
                api.layout_window(head[0], head_rect);
                let tail_rect = tail_rect.split_horizontally(&vec![SplitSpec::Elastic; tail.len()]);
                for (window_id, rect) in izip!(tail, tail_rect) {
                    api.layout_window(window_id, rect);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum LayoutMessageSelect {
    Next,
    Prev,
}

impl LayoutMessageI for LayoutMessageSelect {}

pub struct LayoutNodeSelect {
    node_ids: NonEmptyFocusedVec<Id<LayoutNode>>,
}

impl LayoutNodeSelect {
    pub fn new(node_ids: NonEmptyFocusedVec<Id<LayoutNode>>) -> Self {
        Self { node_ids }
    }
}

impl LayoutNodeI for LayoutNodeSelect {
    fn layout(&self, api: &mut ViewLayoutApi<'_>) {
        let node_id = *self.node_ids.focus();
        api.layout_node(node_id, *api.rect());
    }

    fn handle_message(
        &mut self,
        _api: &mut ViewHandleMessageApi<'_>,
        message: &LayoutMessage,
    ) -> std::ops::ControlFlow<()> {
        let Some(message) = message.downcast_ref::<LayoutMessageSelect>() else {
            return std::ops::ControlFlow::Continue(());
        };

        let count = match message {
            LayoutMessageSelect::Next => 1,
            LayoutMessageSelect::Prev => -1,
        };
        let i = self.node_ids.mod_plus_focused_index(count);
        self.node_ids.set_focused_index(i);

        std::ops::ControlFlow::Break(())
    }
}

#[derive(Debug, Clone)]
pub struct LayoutMessageToggle;

impl LayoutMessageI for LayoutMessageToggle {}

pub struct LayoutNodeToggle {
    node_ids: NonEmptyFocusedVec<Id<LayoutNode>>,
}

impl LayoutNodeToggle {
    pub fn new(node_id_default: Id<LayoutNode>, node_id_toggle: Id<LayoutNode>) -> Self {
        let node_ids = NonEmptyFocusedVec::new(vec![node_id_default, node_id_toggle], 0);
        Self { node_ids }
    }
}

impl LayoutNodeI for LayoutNodeToggle {
    fn layout(&self, api: &mut ViewLayoutApi<'_>) {
        let node_id = *self.node_ids.focus();
        api.layout_node(node_id, *api.rect());
    }

    fn handle_message(
        &mut self,
        _api: &mut ViewHandleMessageApi<'_>,
        message: &LayoutMessage,
    ) -> std::ops::ControlFlow<()> {
        let Some(_) = message.downcast_ref::<LayoutMessageToggle>() else {
            return std::ops::ControlFlow::Continue(());
        };

        let i = self.node_ids.mod_plus_focused_index(1);
        self.node_ids.set_focused_index(i);

        std::ops::ControlFlow::Break(())
    }
}
