use crate::util::Id;
use crate::view::api::ViewLayoutApi;
use crate::view::window::Window;

pub trait LayoutNodeI {
    fn layout(&self, api: &mut ViewLayoutApi);
    fn get_focused_window_id(&self, api: &mut ViewLayoutApi) -> Option<Id<Window>>;
}

pub struct LayoutNode {
    id: Id<Self>,
    inner: Box<dyn LayoutNodeI>,
}

impl<T> From<T> for LayoutNode
where
    T: LayoutNodeI + 'static,
{
    fn from(inner: T) -> Self {
        Self {
            id: Id::new(),
            inner: Box::new(inner),
        }
    }
}

impl LayoutNode {
    pub fn id(&self) -> Id<Self> {
        self.id
    }

    pub fn layout(&self, api: &mut ViewLayoutApi) {
        self.inner.layout(api);
    }

    pub fn get_focused_window_id(&self, api: &mut ViewLayoutApi) -> Option<Id<Window>> {
        self.inner.get_focused_window_id(api)
    }
}
