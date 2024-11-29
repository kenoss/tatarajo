mod props {
    use smithay::utils::{Logical, Rectangle};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Thickness {
        pub top: u32,
        pub right: u32,
        pub bottom: u32,
        pub left: u32,
    }

    impl From<u32> for Thickness {
        fn from(x: u32) -> Self {
            Self {
                top: x,
                right: x,
                bottom: x,
                left: x,
            }
        }
    }

    impl From<(u32, u32)> for Thickness {
        fn from((y, x): (u32, u32)) -> Self {
            Self {
                top: y,
                right: x,
                bottom: y,
                left: x,
            }
        }
    }

    impl From<(u32, u32, u32, u32)> for Thickness {
        fn from((top, right, bottom, left): (u32, u32, u32, u32)) -> Self {
            Self {
                top,
                right,
                bottom,
                left,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct WindowProps {
        pub geometry: Rectangle<i32, Logical>,
    }
}

#[allow(clippy::module_inception)]
mod window {
    use super::props::*;
    use crate::util::Id;
    use itertools::Itertools;
    use smithay::desktop::space::SpaceElement;
    use smithay::utils::{IsAlive, Logical, Physical, Point, Rectangle, Scale};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    // Note that `SpaceElement` almost necessarily requires `Clone + PartialEq` because, for example, for
    // `Space::map_element()`. And some methods is called with `&self` while it should have `&mut self`, e.g.
    // `SpaceElement::set_activate()`. So, we wrap `WindowInner`.
    #[derive(Clone)]
    pub struct Window {
        id: Id<Window>,
        inner: Arc<Mutex<WindowInner>>,
        swindow: smithay::desktop::Window,
    }

    struct WindowInner {
        props: WindowProps,
    }

    impl PartialEq for Window {
        fn eq(&self, other: &Self) -> bool {
            debug_assert_eq!(self.id == other.id, self.swindow == other.swindow);

            self.id == other.id
        }
    }

    impl Eq for Window {}

    impl Window {
        pub fn new(swindow: smithay::desktop::Window) -> Self {
            let geometry = swindow.geometry();
            let inner = WindowInner {
                props: WindowProps { geometry },
            };
            let inner = Arc::new(Mutex::new(inner));
            Self {
                id: Id::new(),
                inner,
                swindow,
            }
        }

        pub fn id(&self) -> Id<Window> {
            self.id
        }

        // TODO: Remove.
        pub fn smithay_window(&self) -> &smithay::desktop::Window {
            &self.swindow
        }

        pub fn toplevel(&self) -> Option<&smithay::wayland::shell::xdg::ToplevelSurface> {
            self.swindow.toplevel()
        }

        pub fn on_commit(&self) {
            self.swindow.on_commit();
        }

        pub fn surface_under<P: Into<Point<f64, Logical>>>(
            &self,
            point: P,
            surface_type: smithay::desktop::WindowSurfaceType,
        ) -> Option<(
            wayland_server::protocol::wl_surface::WlSurface,
            Point<i32, Logical>,
        )> {
            let point = point.into();
            self.swindow.surface_under(point, surface_type)
        }

        pub fn send_frame<T, F>(
            &self,
            output: &smithay::output::Output,
            time: T,
            throttle: Option<Duration>,
            primary_scan_out_output: F,
        ) where
            T: Into<Duration>,
            F: FnMut(
                    &wayland_server::protocol::wl_surface::WlSurface,
                    &smithay::wayland::compositor::SurfaceData,
                ) -> Option<smithay::output::Output>
                + Copy,
        {
            let time = time.into();
            self.swindow
                .send_frame(output, time, throttle, primary_scan_out_output)
        }

        pub fn set_props(&mut self, props: WindowProps) {
            self.inner.lock().unwrap().props = props;
        }
    }

    impl IsAlive for Window {
        fn alive(&self) -> bool {
            self.swindow.alive()
        }
    }

    impl SpaceElement for Window {
        fn geometry(&self) -> Rectangle<i32, Logical> {
            let props = &self.inner.lock().unwrap().props;
            let mut geometry = props.geometry;
            // Ad-hoc: Use of geometry/bbox in smithay is problematic.
            //
            // In smithay, actual geometry/bbox is `space_element.geometry/bbox()` shifted with location of space element, which is
            // updated with `Space::map_element()`. See `InnerEleemnt::geometry()` and `InnerElement::bbox()`. So, we should keep
            // `geometry/bbox.loc` to (0, 0).
            geometry.loc = Point::default();

            geometry
        }

        fn bbox(&self) -> Rectangle<i32, Logical> {
            let props = &self.inner.lock().unwrap().props;
            let mut bbox = props.geometry;
            // Ditto.
            bbox.loc = Point::default();

            bbox
        }

        fn is_in_input_region(&self, point: &Point<f64, Logical>) -> bool {
            self.swindow.is_in_input_region(point)
        }

        fn z_index(&self) -> u8 {
            0
        }

        fn set_activate(&self, activated: bool) {
            self.swindow.set_activate(activated);
        }

        fn output_enter(&self, output: &smithay::output::Output, overlap: Rectangle<i32, Logical>) {
            self.swindow.output_enter(output, overlap);
        }

        fn output_leave(&self, output: &smithay::output::Output) {
            self.swindow.output_leave(output);
        }

        fn refresh(&self) {
            self.swindow.refresh();
        }
    }

    pub(crate) mod as_render_elements {
        use super::*;
        use smithay::backend::renderer::element::solid::SolidColorRenderElement;
        use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
        use smithay::backend::renderer::element::AsRenderElements;
        use smithay::backend::renderer::{ImportAll, ImportMem, Renderer, Texture};

        #[derive(derive_more::From)]
        #[thin_delegate::register]
        pub enum WindowRenderElement<R>
        where
            R: Renderer,
        {
            Window(WaylandSurfaceRenderElement<R>),
            Decoration(SolidColorRenderElement),
        }

        #[thin_delegate::derive_delegate(external_trait_def = crate::external_trait_def::smithay::backend::renderer::element)]
        impl<R> smithay::backend::renderer::element::Element for WindowRenderElement<R>
        where
            R: smithay::backend::renderer::Renderer,
            <R as smithay::backend::renderer::Renderer>::TextureId: 'static,
            R: ImportAll + ImportMem,
        {
        }

        #[thin_delegate::derive_delegate(external_trait_def = crate::external_trait_def::smithay::backend::renderer::element)]
        impl<R> smithay::backend::renderer::element::RenderElement<R> for WindowRenderElement<R>
        where
            R: smithay::backend::renderer::Renderer,
            <R as smithay::backend::renderer::Renderer>::TextureId: 'static,
            R: ImportAll + ImportMem,
        {
        }

        impl<R> AsRenderElements<R> for Window
        where
            R: Renderer + ImportAll + ImportMem,
            <R as Renderer>::TextureId: Texture + 'static,
        {
            type RenderElement = WindowRenderElement<R>;

            fn render_elements<C>(
                &self,
                renderer: &mut R,
                location: Point<i32, Physical>,
                scale: Scale<f64>,
                alpha: f32,
            ) -> Vec<C>
            where
                C: From<Self::RenderElement>,
            {
                AsRenderElements::render_elements(&self.swindow, renderer, location, scale, alpha)
                    .into_iter()
                    .map(C::from)
                    .collect_vec()
            }
        }
    }
}

pub(crate) use props::*;
pub(crate) use window::as_render_elements::*;
pub(crate) use window::*;
