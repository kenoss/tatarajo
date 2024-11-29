use crate::view::api::ViewLayoutApi;
use crate::view::layout_node::LayoutNodeI;

pub struct LayoutFull {}

impl LayoutNodeI for LayoutFull {
    fn layout(&self, api: &mut ViewLayoutApi<'_>) {
        if let Some(&window_id) = api.stackset().workspaces().focus().stack().focus() {
            api.layout_window(window_id, *api.rect());
        }
    }
}
