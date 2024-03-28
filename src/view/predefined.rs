use crate::model::grid_geometry::RectangleExt;
use crate::model::grid_geometry::SplitSpec;
use crate::util::Id;
use crate::util::NonEmptyFocusedVec;
use crate::view::api::ViewLayoutApi;
use crate::view::layout_node::{LayoutNode, LayoutNodeI};
use crate::view::window::Window;
pub use itertools::izip;

pub struct LayoutNodeStackSet {}

impl LayoutNodeI for LayoutNodeStackSet {
    fn layout(&self, api: &mut ViewLayoutApi) {
        let id = *api.stackset().workspaces().focus().layouts().focus();
        api.layout_node(id, *api.rect())
    }

    fn get_focused_window_id(&self, api: &mut ViewLayoutApi) -> Option<Id<Window>> {
        let id = *api.stackset().workspaces().focus().layouts().focus();
        api.get_focused_window_id(id)
    }
}

pub struct LayoutFull {}

impl LayoutNodeI for LayoutFull {
    fn layout(&self, api: &mut ViewLayoutApi) {
        if let Some(&window_id) = api.stackset().workspaces().focus().stack().focus() {
            api.layout_window(window_id, *api.rect());
        }
    }

    fn get_focused_window_id(&self, api: &mut ViewLayoutApi) -> Option<Id<Window>> {
        api.stackset().workspaces().focus().stack().focus().copied()
    }
}

pub struct LayoutTall {}

impl LayoutNodeI for LayoutTall {
    fn layout(&self, api: &mut ViewLayoutApi) {
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

    fn get_focused_window_id(&self, api: &mut ViewLayoutApi) -> Option<Id<Window>> {
        api.stackset().workspaces().focus().stack().focus().copied()
    }
}

pub struct LayoutNodeSelect {
    node_ids: NonEmptyFocusedVec<Id<LayoutNode>>,
}

impl LayoutNodeSelect {
    pub fn new(node_ids: NonEmptyFocusedVec<Id<LayoutNode>>) -> Self {
        Self { node_ids }
    }
}

impl LayoutNodeI for LayoutNodeSelect {
    fn layout(&self, api: &mut ViewLayoutApi) {
        let node_id = *self.node_ids.focus();
        api.layout_node(node_id, *api.rect());
    }

    fn get_focused_window_id(&self, api: &mut ViewLayoutApi) -> Option<Id<Window>> {
        let node_id = *self.node_ids.focus();
        api.get_focused_window_id(node_id)
    }
}
