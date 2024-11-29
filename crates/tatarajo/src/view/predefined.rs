use crate::model::grid_geometry::{RectangleExt, SplitSpec};
use crate::view::api::ViewLayoutApi;
use crate::view::layout_node::LayoutNodeI;
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
