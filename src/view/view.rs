use crate::model::grid_geometry::RectangleExt;
use crate::util::Id;
use crate::util::NonEmptyFocusedVec;
use crate::view::api::{ViewHandleMessageApi, ViewLayoutApi};
use crate::view::layout_node::LayoutMessage;
use crate::view::layout_node::LayoutNode;
use crate::view::predefined::{LayoutFull, LayoutNodeSelect, LayoutTall};
use crate::view::stackset::StackSet;
use crate::view::window::{Border, Rgba, Window, WindowProps};
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
    pub(super) root_node_id: Id<LayoutNode>,
    pub(super) rect: Rectangle<i32, Logical>,
}

impl View {
    pub fn new(rect: Rectangle<i32, Logical>) -> Self {
        let mut nodes = HashMap::new();

        let node = LayoutNode::from(LayoutTall {});
        let node_id0 = node.id();
        nodes.insert(node_id0, RefCell::new(node));

        let node = LayoutNode::from(LayoutFull {});
        let node_id1 = node.id();
        nodes.insert(node_id1, RefCell::new(node));

        let layouts = NonEmptyFocusedVec::new(vec![node_id0, node_id1], 0);
        let node = LayoutNode::from(LayoutNodeSelect::new(layouts));
        let node_id = node.id();
        nodes.insert(node_id, RefCell::new(node));

        let stackset = StackSet::new();

        let state = ViewState {
            stackset,
            nodes,
            layout_queue: VecDeque::new(),
            windows: HashMap::new(),
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

    pub fn layout(&mut self, space: &mut smithay::desktop::Space<Window>) {
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

        // Reflect layout to the space and surfaces.
        while let Some((window_id, rect)) = self.state.layout_queue.pop_front() {
            let margin = 8.into();
            let border = Border {
                dim: 2.into(),
                active_rgba: Rgba::from_rgba(0x556b2fff),
                inactive_rgba: Rgba::from_rgba(0x00000000),
            };
            let rect = rect.shrink(margin).shrink(border.dim.clone());
            let props = WindowProps {
                geometry: rect,
                border,
            };
            let window = self.state.windows.get_mut(&window_id).unwrap();
            window.set_props(props);
            space.map_element(window.clone(), rect.loc, false);
            let Some(surface) = window.toplevel() else {
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
        }

        assert!(self.state.layout_queue.is_empty());
    }

    pub fn handle_layout_message(
        &mut self,
        message: &LayoutMessage,
        space: &mut smithay::desktop::Space<Window>,
    ) {
        let root_node_id = self.state.root_node_id;
        let mut api = ViewHandleMessageApi {
            state: &mut self.state,
        };
        api.handle_message(root_node_id, message);

        self.layout(space);
    }

    pub fn resize_output(
        &mut self,
        size: Size<i32, Logical>,
        space: &mut smithay::desktop::Space<Window>,
    ) {
        self.state.rect = Rectangle::from_loc_and_size((0, 0), size);
        self.layout(space);
    }

    pub fn register_window(&mut self, smithay_window: smithay::desktop::Window) -> Id<Window> {
        let window = Window::new(smithay_window);
        let window_id = window.id();
        self.state
            .stackset
            .workspaces
            .focus_mut()
            .stack
            .push(window_id);
        self.state.windows.insert(window_id, window);

        window_id
    }

    pub fn set_focus(&mut self, id: Id<Window>) {
        let workspaces = &mut self.state.stackset.workspaces;

        let mut indice = None;
        for (i, ws) in workspaces.as_vec().iter().enumerate() {
            for (j, &window_id) in ws.stack.as_vec().iter().enumerate() {
                if window_id == id {
                    indice = Some((i, j));
                    break;
                }
            }
        }
        let Some((i, j)) = indice else {
            return;
        };

        workspaces.set_focused_index(i);
        workspaces.focus_mut().stack.set_focused_index(j);
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
