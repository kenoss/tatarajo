use crate::util::Id;
use crate::view::api::{ViewHandleMessageApi, ViewLayoutApi};
use downcast::Any;
use dyn_clone::DynClone;

pub trait LayoutMessageI: Any + std::fmt::Debug + DynClone {}

downcast::downcast!(dyn LayoutMessageI);
dyn_clone::clone_trait_object!(LayoutMessageI);

#[derive(Debug, Clone)]
pub struct LayoutMessage {
    inner: Box<dyn LayoutMessageI>,
}

impl<T> From<T> for LayoutMessage
where
    T: LayoutMessageI,
{
    fn from(x: T) -> Self {
        Self { inner: Box::new(x) }
    }
}

impl LayoutMessage {
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: LayoutMessageI,
    {
        self.inner.as_ref().downcast_ref().ok()
    }
}

pub trait LayoutNodeI {
    fn layout(&self, api: &mut ViewLayoutApi<'_>);

    // The defalut implementation is for leaf node.
    fn handle_message(
        &mut self,
        _api: &mut ViewHandleMessageApi<'_>,
        _message: &LayoutMessage,
    ) -> std::ops::ControlFlow<()> {
        std::ops::ControlFlow::Continue(())
    }
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

    pub fn handle_message(
        &mut self,
        api: &mut ViewHandleMessageApi<'_>,
        message: &LayoutMessage,
    ) -> std::ops::ControlFlow<()> {
        self.inner.handle_message(api, message)
    }
}
