use crate::drawing::{PointerRenderElement, CLEAR_COLOR};
use crate::view::window::WindowRenderElement;
use smithay::backend::renderer::damage::{
    Error as OutputDamageTrackerError, OutputDamageTracker, RenderOutputResult,
};
use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::element::{RenderElement, Wrap};
use smithay::backend::renderer::{ImportAll, ImportMem, Renderer};
use smithay::desktop::space::{Space, SpaceRenderElements};
use smithay::output::Output;

smithay::backend::renderer::element::render_elements! {
    pub CustomRenderElements<R> where
        R: ImportAll + ImportMem;
    Pointer=PointerRenderElement<R>,
    Surface=WaylandSurfaceRenderElement<R>,
}

impl<R> std::fmt::Debug for CustomRenderElements<R>
where
    R: Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pointer(arg0) => f.debug_tuple("Pointer").field(arg0).finish(),
            Self::Surface(arg0) => f.debug_tuple("Surface").field(arg0).finish(),
            Self::_GenericCatcher(arg0) => f.debug_tuple("_GenericCatcher").field(arg0).finish(),
        }
    }
}

smithay::backend::renderer::element::render_elements! {
    pub OutputRenderElements<R, E> where R: ImportAll + ImportMem;
    Space=SpaceRenderElements<R, E>,
    Window=Wrap<E>,
    Custom=CustomRenderElements<R>,
}

impl<R, E> std::fmt::Debug for OutputRenderElements<R, E>
where
    R: Renderer + ImportAll + ImportMem,
    E: RenderElement<R> + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Space(arg0) => f.debug_tuple("Space").field(arg0).finish(),
            Self::Window(arg0) => f.debug_tuple("Window").field(arg0).finish(),
            Self::Custom(arg0) => f.debug_tuple("Custom").field(arg0).finish(),
            Self::_GenericCatcher(arg0) => f.debug_tuple("_GenericCatcher").field(arg0).finish(),
        }
    }
}

pub fn output_elements<R>(
    output: &Output,
    space: &Space<crate::view::window::Window>,
    custom_elements: impl IntoIterator<Item = CustomRenderElements<R>>,
    renderer: &mut R,
) -> (
    Vec<OutputRenderElements<R, WindowRenderElement<R>>>,
    [f32; 4],
)
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + 'static,
{
    let mut output_render_elements = custom_elements
        .into_iter()
        .map(OutputRenderElements::from)
        .collect::<Vec<_>>();

    let space_elements = smithay::desktop::space::space_render_elements::<
        _,
        crate::view::window::Window,
        _,
    >(renderer, [space], output, 1.0)
    .expect("output without mode?");
    output_render_elements.extend(space_elements.into_iter().map(OutputRenderElements::Space));

    (output_render_elements, CLEAR_COLOR)
}

#[allow(clippy::too_many_arguments)]
pub fn render_output<R>(
    output: &Output,
    space: &Space<crate::view::window::Window>,
    custom_elements: impl IntoIterator<Item = CustomRenderElements<R>>,
    renderer: &mut R,
    damage_tracker: &mut OutputDamageTracker,
    age: usize,
) -> Result<RenderOutputResult, OutputDamageTrackerError<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + 'static,
{
    let (elements, clear_color) = output_elements(output, space, custom_elements, renderer);
    damage_tracker.render_output(renderer, age, &elements, clear_color)
}
