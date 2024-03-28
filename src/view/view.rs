use crate::model::grid_geometry::RectangleExt;
use crate::util::Id;
use crate::util::NonEmptyFocusedVec;
use crate::view::api::ViewLayoutApi;
use crate::view::layout_node::LayoutNode;
use crate::view::predefined::{LayoutNodeSelect, LayoutNodeStackSet, LayoutTall};
use crate::view::stackset::StackSet;
use crate::view::window::Window;
use smithay::utils::{Logical, Rectangle, Size};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

pub struct View {
    // TODO: Avoid internal struct if possible.
    state: ViewState,
}

pub(super) struct ViewState {
    pub(super) stackset: StackSet,
    pub(super) nodes: HashMap<Id<LayoutNode>, RefCell<LayoutNode>>,
    // TODO: Rename.
    pub(super) layout_queue: VecDeque<(Id<Window>, Rectangle<i32, Logical>)>,
    pub(super) windows: HashMap<Id<Window>, Window>,
    pub(super) smithay_windows: HashMap<Id<Window>, smithay::desktop::Window>,
    pub(super) root_node_id: Id<LayoutNode>,
    pub(super) rect: Rectangle<i32, Logical>,
}

impl View {
    pub fn new(rect: Rectangle<i32, Logical>) -> Self {
        let mut nodes = HashMap::new();
        let windows = HashMap::new();

        let node = LayoutNode::from(LayoutTall {});
        let node_id = node.id();
        let layouts = vec![node_id];
        let stackset = StackSet::new(layouts);
        nodes.insert(node_id, RefCell::new(node));

        let node = LayoutNode::from(LayoutNodeStackSet {});
        let node_id = node.id();
        nodes.insert(node_id, RefCell::new(node));

        let layouts = NonEmptyFocusedVec::new(vec![node_id], 0);
        let node = LayoutNode::from(LayoutNodeSelect::new(layouts));
        let node_id = node.id();
        nodes.insert(node_id, RefCell::new(node));

        let state = ViewState {
            stackset,
            nodes,
            layout_queue: VecDeque::new(),
            windows,
            smithay_windows: HashMap::new(),
            root_node_id: node_id,
            rect,
        };
        Self { state }
    }

    pub fn startup(&mut self) {
        // TODO: Remove?
    }

    pub fn stackset(&self) -> &StackSet {
        &self.state.stackset
    }

    pub fn window(&self, window_id: Id<Window>) -> Option<&Window> {
        self.state.windows.get(&window_id)
    }

    pub fn layout(&mut self, space: &mut smithay::desktop::Space<smithay::desktop::Window>) {
        use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;

        assert!(self.state.layout_queue.is_empty());

        // Layout
        let root_node_id = self.state.root_node_id;
        let rect = self.state.rect;
        let mut api = ViewLayoutApi {
            state: &mut self.state,
            rect,
        };
        api.layout_node(root_node_id, rect);

        while let Some((window_id, rect)) = self.state.layout_queue.pop_front() {
            let rect = rect.shrink((8, 8, 8, 8));
            let smithay_window = self.state.smithay_windows.get(&window_id).unwrap();
            let Some(surface) = smithay_window.toplevel() else {
                continue;
            };
            surface.with_pending_state(|state| {
                state.states.set(xdg_toplevel::State::Fullscreen);
                state.states.set(xdg_toplevel::State::TiledTop);
                state.states.set(xdg_toplevel::State::TiledLeft);
                state.states.set(xdg_toplevel::State::TiledBottom);
                state.states.set(xdg_toplevel::State::TiledRight);
                state.size = Some(rect.size);
            });
            surface.send_pending_configure();
            space.map_element(smithay_window.clone(), rect.loc, false);
        }

        assert!(self.state.layout_queue.is_empty());
    }

    pub fn resize_output(
        &mut self,
        size: Size<i32, Logical>,
        space: &mut smithay::desktop::Space<smithay::desktop::Window>,
    ) {
        self.state.rect = Rectangle::from_loc_and_size((0, 0), size);
        self.layout(space);
    }

    pub fn register_window(&mut self, smithay_window: smithay::desktop::Window) {
        let window = Window::new();
        self.state
            .stackset
            .workspaces
            .focus_mut()
            .stack
            .push(window.id());
        self.state
            .smithay_windows
            .insert(window.id(), smithay_window);
        self.state.windows.insert(window.id(), window);
    }

    pub fn focused_window(&self) -> Option<&Window> {
        self.state
            .stackset
            .workspaces
            .focus()
            .stack
            .focus()
            .map(|id| self.state.windows.get(id).unwrap())
    }

    pub fn focus_next_window(&mut self, count: isize) {
        self.state
            .stackset
            .workspaces
            .focus_mut()
            .focus_next_window(count);
    }
}
