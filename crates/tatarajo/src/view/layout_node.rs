use crate::util::Id;
use crate::view::api::ViewLayoutApi;

pub trait LayoutNodeI {
    fn layout(&self, api: &mut ViewLayoutApi<'_>);
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

    pub fn layout(&self, api: &mut ViewLayoutApi<'_>) {
        self.inner.layout(api);
    }
}
