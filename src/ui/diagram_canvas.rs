use crate::design_system::tokens::ColorTokens;
use crate::features::concept_model::{
    DiagramEdge, DiagramNode, NodeId, PortSide, RelationKind, GRID_SIZE,
};
use crate::features::information_model::{ClassDiagramEdge, ClassDiagramNode};
use crate::ui::edge_router::EdgeRouter;
use crate::ui::theme::ThemeColors;
use iced::mouse;
use iced::widget::canvas::{Action, Event, Frame, Geometry, Path, Program, Stroke, Text};
use iced::{alignment, Color, Point, Rectangle, Renderer, Size, Theme, Vector};
use std::time::Instant;

/// Fælles viewport til koordinattransformation, zoom og panorering
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasViewport {
    pan: Vector,
    zoom: f32,
}

impl Default for CanvasViewport {
    fn default() -> Self {
        Self {
            pan: Vector::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl CanvasViewport {
    pub const MIN_ZOOM: f32 = 0.25;
    pub const MAX_ZOOM: f32 = 1.50;

    pub fn new(pan: Vector, zoom: f32) -> Self {
        let mut vp = Self { pan, zoom: 1.0 };
        vp.set_zoom(zoom);
        vp
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
    }

    pub fn pan(&self) -> Vector {
        self.pan
    }

    pub fn set_pan(&mut self, pan: Vector) {
        self.pan = pan;
    }

    pub fn translate(&mut self, delta: Vector) {
        self.pan += delta;
    }

    pub fn to_world(&self, screen_point: Point) -> Point {
        Point::new(
            (screen_point.x - self.pan.x) / self.zoom,
            (screen_point.y - self.pan.y) / self.zoom,
        )
    }

    pub fn to_screen(&self, world_point: Point) -> Point {
        Point::new(
            world_point.x * self.zoom + self.pan.x,
            world_point.y * self.zoom + self.pan.y,
        )
    }

    pub fn zoom_at(&mut self, screen_pivot: Point, factor: f32) {
        let world_pivot = self.to_world(screen_pivot);
        let new_zoom = (self.zoom * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        if (new_zoom - self.zoom).abs() < f32::EPSILON {
            return;
        }
        self.zoom = new_zoom;
        self.pan = Vector::new(
            screen_pivot.x - world_pivot.x * self.zoom,
            screen_pivot.y - world_pivot.y * self.zoom,
        );
    }

    pub fn snap_to_grid(world_point: Point, grid_size: f32) -> Point {
        Point::new(
            (world_point.x / grid_size).round() * grid_size,
            (world_point.y / grid_size).round() * grid_size,
        )
    }

    pub fn reset(&mut self) {
        self.pan = Vector::new(0.0, 0.0);
        self.zoom = 1.0;
    }
}

pub const CLICK_SLOP_PX: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickTarget {
    Node(NodeId),
    Empty,
}

#[derive(Debug, Clone, Copy)]
pub struct ClickRecord {
    pub position: Point,
    pub world_position: Point,
    pub time: Instant,
    pub target: ClickTarget,
}

/// Abstraktion for noder på et diagram
pub trait CanvasNode {
    fn id(&self) -> NodeId;
    fn position(&self) -> (f32, f32);
    fn size(&self) -> (f32, f32);
    fn contains(&self, px: f32, py: f32) -> bool {
        let (x, y) = self.position();
        let (w, h) = self.size();
        px >= x && px <= x + w && py >= y && py <= y + h
    }
    fn to_diagram_node(&self) -> DiagramNode {
        let (x, y) = self.position();
        let (w, h) = self.size();
        DiagramNode::custom(self.id(), String::new(), x, y, w, h)
    }
}

/// Abstraktion for kanter/relationer på et diagram
pub trait CanvasEdge {
    fn from(&self) -> NodeId;
    fn to(&self) -> NodeId;
    fn kind(&self) -> RelationKind;
    fn label(&self) -> Option<&str>;
    fn source_port(&self) -> Option<PortSide> {
        None
    }
    fn target_port(&self) -> Option<PortSide> {
        None
    }
    fn source_multiplicity(&self) -> Option<String> {
        None
    }
    fn target_multiplicity(&self) -> Option<String> {
        None
    }
    fn is_directed(&self) -> bool {
        true
    }
    fn to_diagram_edge(&self) -> DiagramEdge {
        let mut edge = DiagramEdge::with_ports(
            self.from(),
            self.to(),
            self.kind(),
            self.label().map(|s| s.to_string()),
            self.source_port(),
            self.target_port(),
        );
        edge.set_directed(self.is_directed());
        edge
    }
}

impl CanvasNode for DiagramNode {
    fn id(&self) -> NodeId {
        self.id()
    }
    fn position(&self) -> (f32, f32) {
        (self.x(), self.y())
    }
    fn size(&self) -> (f32, f32) {
        (self.width(), self.height())
    }
    fn contains(&self, px: f32, py: f32) -> bool {
        DiagramNode::contains(self, px, py)
    }
}

impl CanvasNode for ClassDiagramNode {
    fn id(&self) -> NodeId {
        self.id()
    }
    fn position(&self) -> (f32, f32) {
        (self.x(), self.y())
    }
    fn size(&self) -> (f32, f32) {
        (self.width(), self.height())
    }
    fn contains(&self, px: f32, py: f32) -> bool {
        ClassDiagramNode::contains(self, px, py)
    }
}

impl CanvasEdge for DiagramEdge {
    fn from(&self) -> NodeId {
        self.from()
    }
    fn to(&self) -> NodeId {
        self.to()
    }
    fn kind(&self) -> RelationKind {
        self.kind()
    }
    fn label(&self) -> Option<&str> {
        self.label()
    }
    fn source_port(&self) -> Option<PortSide> {
        self.source_port()
    }
    fn target_port(&self) -> Option<PortSide> {
        self.target_port()
    }
    fn is_directed(&self) -> bool {
        DiagramEdge::is_directed(self)
    }
}

impl CanvasEdge for ClassDiagramEdge {
    fn from(&self) -> NodeId {
        self.from()
    }
    fn to(&self) -> NodeId {
        self.to()
    }
    fn kind(&self) -> RelationKind {
        self.kind()
    }
    fn label(&self) -> Option<&str> {
        self.label()
    }
    fn source_port(&self) -> Option<PortSide> {
        self.source_port()
    }
    fn target_port(&self) -> Option<PortSide> {
        self.target_port()
    }
    fn is_directed(&self) -> bool {
        ClassDiagramEdge::is_directed(self)
    }
    fn source_multiplicity(&self) -> Option<String> {
        if self.kind() == RelationKind::Generalization {
            None
        } else {
            ClassDiagramEdge::source_multiplicity(self).map(|m| m.to_display_string())
        }
    }
    fn target_multiplicity(&self) -> Option<String> {
        if self.kind() == RelationKind::Generalization {
            None
        } else {
            ClassDiagramEdge::target_multiplicity(self).map(|m| m.to_display_string())
        }
    }
}

/// Tilstand for rektangulær drag-select (marquee)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarqueeState {
    pub start: Point,
    pub current: Point,
}

/// Tilstand for synkron parallelforskydning af noder (bulk drag)
#[derive(Debug, Clone, PartialEq)]
pub struct BulkDragState {
    pub leader_id: NodeId,
    pub start_world_pos: Point,
    pub initial_positions: Vec<(NodeId, (f32, f32))>,
    pub last_delta: Option<(f32, f32)>,
    pub start_time: std::time::Instant,
}

#[derive(Debug, Default)]
pub struct DiagramCanvasState {
    pub dragging_node: Option<(NodeId, Vector)>,
    pub bulk_drag: Option<BulkDragState>,
    pub marquee: Option<MarqueeState>,
    pub last_click: Option<ClickRecord>,
    pub pending_double: Option<(ClickTarget, Point)>,
    pub press_screen_pos: Option<Point>,
    pub panning_start: Option<(Point, Vector)>,
    pub is_panning_space: bool,
    pub is_panning_minimap: bool,
    pub modifiers: iced::keyboard::Modifiers,
    pub connecting_from: Option<NodeId>,
    pub connecting_cursor: Option<Point>,
    pub hovered_target_node: Option<NodeId>,
    pub selected_node_ids: std::collections::HashSet<NodeId>,
    pub last_canvas_selection: Option<std::collections::HashSet<NodeId>>,
}

/// Koordinattransformation for miniaturekort (Minimap)
#[derive(Debug, Clone, Copy)]
pub struct MinimapTransform {
    pub world_bounds: Rectangle,
    pub minimap_rect: Rectangle,
    pub scale: f32,
    pub offset: Point,
}

impl MinimapTransform {
    pub fn compute<N: CanvasNode>(
        nodes: &[N],
        viewport: &CanvasViewport,
        screen_bounds: Size,
        minimap_rect: Rectangle,
    ) -> Self {
        let vp_tl = viewport.to_world(Point::ORIGIN);
        let vp_br = viewport.to_world(Point::new(screen_bounds.width, screen_bounds.height));

        let mut min_x = vp_tl.x.min(vp_br.x);
        let mut max_x = vp_tl.x.max(vp_br.x);
        let mut min_y = vp_tl.y.min(vp_br.y);
        let mut max_y = vp_tl.y.max(vp_br.y);

        for node in nodes {
            let (nx, ny) = node.position();
            let (nw, nh) = node.size();
            min_x = min_x.min(nx);
            min_y = min_y.min(ny);
            max_x = max_x.max(nx + nw);
            max_y = max_y.max(ny + nh);
        }

        let margin = 60.0;
        min_x -= margin;
        min_y -= margin;
        max_x += margin;
        max_y += margin;

        let world_w = (max_x - min_x).max(100.0);
        let world_h = (max_y - min_y).max(100.0);

        let scale_x = minimap_rect.width / world_w;
        let scale_y = minimap_rect.height / world_h;
        let scale = scale_x.min(scale_y);

        let content_w = world_w * scale;
        let content_h = world_h * scale;
        let offset_x = minimap_rect.x + (minimap_rect.width - content_w) / 2.0;
        let offset_y = minimap_rect.y + (minimap_rect.height - content_h) / 2.0;

        Self {
            world_bounds: Rectangle::new(Point::new(min_x, min_y), Size::new(world_w, world_h)),
            minimap_rect,
            scale,
            offset: Point::new(offset_x, offset_y),
        }
    }

    pub fn world_to_minimap(&self, world_pt: Point) -> Point {
        Point::new(
            self.offset.x + (world_pt.x - self.world_bounds.x) * self.scale,
            self.offset.y + (world_pt.y - self.world_bounds.y) * self.scale,
        )
    }

    pub fn minimap_to_world(&self, minimap_pt: Point) -> Point {
        Point::new(
            self.world_bounds.x + (minimap_pt.x - self.offset.x) / self.scale,
            self.world_bounds.y + (minimap_pt.y - self.offset.y) / self.scale,
        )
    }

    pub fn viewport_rect(&self, viewport: &CanvasViewport, screen_bounds: Size) -> Rectangle {
        let vp_tl = viewport.to_world(Point::ORIGIN);
        let m_tl = self.world_to_minimap(vp_tl);
        let m_w = (screen_bounds.width / viewport.zoom()) * self.scale;
        let m_h = (screen_bounds.height / viewport.zoom()) * self.scale;
        Rectangle::new(m_tl, Size::new(m_w, m_h))
    }
}

/// Beregner optimal viewport for at centrere og fitte alle noder på lærredet
pub fn compute_fit_to_view<N: CanvasNode>(nodes: &[N], screen_bounds: Size) -> CanvasViewport {
    if nodes.is_empty() {
        return CanvasViewport::default();
    }
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for n in nodes {
        let (nx, ny) = n.position();
        let (nw, nh) = n.size();
        min_x = min_x.min(nx);
        min_y = min_y.min(ny);
        max_x = max_x.max(nx + nw);
        max_y = max_y.max(ny + nh);
    }
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let world_w = (max_x - min_x).max(100.0) + 120.0;
    let world_h = (max_y - min_y).max(100.0) + 120.0;

    let zoom_x = screen_bounds.width / world_w;
    let zoom_y = screen_bounds.height / world_h;
    let target_zoom = zoom_x.min(zoom_y).clamp(CanvasViewport::MIN_ZOOM, 1.0);

    let pan = Vector::new(
        screen_bounds.width / 2.0 - center_x * target_zoom,
        screen_bounds.height / 2.0 - center_y * target_zoom,
    );
    CanvasViewport::new(pan, target_zoom)
}

/// Afstand fra punkt p til linjesegment a-b
fn distance_to_segment(p: Point, a: Point, b: Point) -> f32 {
    let l2 = (b.x - a.x).powi(2) + (b.y - a.y).powi(2);
    if l2 < 0.0001 {
        return (p.x - a.x).hypot(p.y - a.y);
    }
    let t = (((p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y)) / l2).clamp(0.0, 1.0);
    let projection = Point::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y));
    (p.x - projection.x).hypot(p.y - projection.y)
}

type EdgeSelectHandler<'a, Message> = Box<dyn Fn(Option<(NodeId, NodeId)>) -> Message + 'a>;
type EdgeCreateHandler<'a, Message> = Box<dyn Fn(NodeId, NodeId) -> Message + 'a>;
type SelectionChangeHandler<'a, Message> =
    Box<dyn Fn(std::collections::HashSet<NodeId>) -> Message + 'a>;
type NodesMovedHandler<'a, Message> = Box<dyn Fn(Vec<(NodeId, f32, f32)>) -> Message + 'a>;

/// Unificeret DiagramCanvas-komponent med pluggable node-rendering
pub struct DiagramCanvas<'a, Message, N, E, R>
where
    N: CanvasNode,
    E: CanvasEdge,
    R: Fn(&mut Frame, &N, bool, CanvasViewport),
{
    nodes: &'a [N],
    edges: &'a [E],
    selected_node_id: Option<NodeId>,
    selected_node_ids: std::collections::HashSet<NodeId>,
    selected_edge: Option<(NodeId, NodeId)>,
    viewport: CanvasViewport,
    is_space_pressed: bool,
    is_palette_dragging: bool,
    render_node: R,
    on_node_selected: Box<dyn Fn(Option<NodeId>) -> Message + 'a>,
    on_selection_changed: Option<SelectionChangeHandler<'a, Message>>,
    on_edge_selected: Option<EdgeSelectHandler<'a, Message>>,
    on_edge_created: Option<EdgeCreateHandler<'a, Message>>,
    on_node_moved: Box<dyn Fn(NodeId, f32, f32) -> Message + 'a>,
    on_nodes_moved: Option<NodesMovedHandler<'a, Message>>,
    on_canvas_double_clicked: Box<dyn Fn(f32, f32) -> Message + 'a>,
    on_node_double_clicked: Box<dyn Fn(NodeId) -> Message + 'a>,
    on_viewport_changed: Box<dyn Fn(CanvasViewport) -> Message + 'a>,
    on_canvas_drop: Option<Box<dyn Fn(f32, f32) -> Message + 'a>>,
    on_drag_released: Option<Box<dyn Fn() -> Message + 'a>>,
}

impl<'a, Message, N, E, R> DiagramCanvas<'a, Message, N, E, R>
where
    N: CanvasNode,
    E: CanvasEdge,
    R: Fn(&mut Frame, &N, bool, CanvasViewport) + 'a,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        nodes: &'a [N],
        edges: &'a [E],
        selected_node_id: Option<NodeId>,
        viewport: CanvasViewport,
        is_space_pressed: bool,
        render_node: R,
        on_node_selected: impl Fn(Option<NodeId>) -> Message + 'a,
        on_node_moved: impl Fn(NodeId, f32, f32) -> Message + 'a,
        on_canvas_double_clicked: impl Fn(f32, f32) -> Message + 'a,
        on_node_double_clicked: impl Fn(NodeId) -> Message + 'a,
        on_viewport_changed: impl Fn(CanvasViewport) -> Message + 'a,
    ) -> Self {
        Self {
            nodes,
            edges,
            selected_node_id,
            selected_node_ids: selected_node_id.into_iter().collect(),
            selected_edge: None,
            viewport,
            is_space_pressed,
            is_palette_dragging: false,
            render_node,
            on_node_selected: Box::new(on_node_selected),
            on_selection_changed: None,
            on_edge_selected: None,
            on_edge_created: None,
            on_node_moved: Box::new(on_node_moved),
            on_nodes_moved: None,
            on_canvas_double_clicked: Box::new(on_canvas_double_clicked),
            on_node_double_clicked: Box::new(on_node_double_clicked),
            on_viewport_changed: Box::new(on_viewport_changed),
            on_canvas_drop: None,
            on_drag_released: None,
        }
    }

    pub fn on_drag_released(mut self, handler: impl Fn() -> Message + 'a) -> Self {
        self.on_drag_released = Some(Box::new(handler));
        self
    }

    pub fn is_palette_dragging(mut self, is_dragging: bool) -> Self {
        self.is_palette_dragging = is_dragging;
        self
    }

    pub fn on_canvas_drop(mut self, handler: impl Fn(f32, f32) -> Message + 'a) -> Self {
        self.on_canvas_drop = Some(Box::new(handler));
        self
    }

    pub fn selected_node_ids(mut self, ids: impl IntoIterator<Item = NodeId>) -> Self {
        self.selected_node_ids = ids.into_iter().collect();
        if self.selected_node_id.is_none() {
            self.selected_node_id = self.selected_node_ids.iter().next().copied();
        }
        self
    }

    pub fn on_selection_changed(
        mut self,
        handler: impl Fn(std::collections::HashSet<NodeId>) -> Message + 'a,
    ) -> Self {
        self.on_selection_changed = Some(Box::new(handler));
        self
    }

    pub fn on_nodes_moved(
        mut self,
        handler: impl Fn(Vec<(NodeId, f32, f32)>) -> Message + 'a,
    ) -> Self {
        self.on_nodes_moved = Some(Box::new(handler));
        self
    }

    pub fn is_node_selected(&self, id: NodeId) -> bool {
        self.selected_node_ids.contains(&id) || self.selected_node_id == Some(id)
    }

    pub fn selected_edge(mut self, edge: Option<(NodeId, NodeId)>) -> Self {
        self.selected_edge = edge;
        self
    }

    pub fn on_edge_selected(
        mut self,
        handler: impl Fn(Option<(NodeId, NodeId)>) -> Message + 'a,
    ) -> Self {
        self.on_edge_selected = Some(Box::new(handler));
        self
    }

    pub fn on_edge_created(mut self, handler: impl Fn(NodeId, NodeId) -> Message + 'a) -> Self {
        self.on_edge_created = Some(Box::new(handler));
        self
    }

    pub const PANEL_WIDTH: f32 = 180.0;
    pub const PANEL_HEIGHT: f32 = 148.0;
    pub const PANEL_MARGIN: f32 = 16.0;
    pub const MINIMAP_HEIGHT: f32 = 104.0;
    pub const TOOLBAR_HEIGHT: f32 = 32.0;

    pub fn floating_panel_rect(bounds: Rectangle) -> Rectangle {
        let x = bounds.width - Self::PANEL_WIDTH - Self::PANEL_MARGIN;
        let y = bounds.height - Self::PANEL_HEIGHT - Self::PANEL_MARGIN;
        Rectangle::new(
            Point::new(x, y),
            Size::new(Self::PANEL_WIDTH, Self::PANEL_HEIGHT),
        )
    }

    pub fn minimap_rect(panel_rect: Rectangle) -> Rectangle {
        Rectangle::new(
            Point::new(panel_rect.x + 6.0, panel_rect.y + 6.0),
            Size::new(panel_rect.width - 12.0, Self::MINIMAP_HEIGHT),
        )
    }

    pub fn zoom_out_button_rect(panel_rect: Rectangle) -> Rectangle {
        Rectangle::new(
            Point::new(panel_rect.x + 8.0, panel_rect.y + 116.0),
            Size::new(26.0, 24.0),
        )
    }

    pub fn zoom_label_rect(panel_rect: Rectangle) -> Rectangle {
        Rectangle::new(
            Point::new(panel_rect.x + 38.0, panel_rect.y + 116.0),
            Size::new(48.0, 24.0),
        )
    }

    pub fn zoom_in_button_rect(panel_rect: Rectangle) -> Rectangle {
        Rectangle::new(
            Point::new(panel_rect.x + 90.0, panel_rect.y + 116.0),
            Size::new(26.0, 24.0),
        )
    }

    pub fn fit_view_button_rect(panel_rect: Rectangle) -> Rectangle {
        Rectangle::new(
            Point::new(panel_rect.x + 120.0, panel_rect.y + 116.0),
            Size::new(52.0, 24.0),
        )
    }
}

impl<'a, Message, N, E, R> Program<Message, Theme, Renderer> for DiagramCanvas<'a, Message, N, E, R>
where
    N: CanvasNode,
    E: CanvasEdge,
    R: Fn(&mut Frame, &N, bool, CanvasViewport) + 'a,
{
    type State = DiagramCanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        if state.last_canvas_selection.as_ref() != Some(&self.selected_node_ids) {
            state.selected_node_ids = self.selected_node_ids.clone();
            state.last_canvas_selection = Some(self.selected_node_ids.clone());
        }

        let cursor_pos = cursor.position_in(bounds)?;

        match event {
            Event::Keyboard(iced::keyboard::Event::KeyPressed { modifiers, .. }) => {
                state.modifiers = *modifiers;
                None
            }
            Event::Keyboard(iced::keyboard::Event::KeyReleased { modifiers, .. }) => {
                state.modifiers = *modifiers;
                None
            }
            Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
                state.modifiers = *modifiers;
                None
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let (dx, dy) = match delta {
                    mouse::ScrollDelta::Lines { x, y } => (*x * 30.0, *y * 30.0),
                    mouse::ScrollDelta::Pixels { x, y } => (*x, *y),
                };

                if state.modifiers.control() || state.modifiers.command() {
                    let zoom_factor: f32 = if dy > 0.0 {
                        1.1
                    } else if dy < 0.0 {
                        1.0 / 1.1
                    } else {
                        1.0
                    };
                    if (zoom_factor - 1.0f32).abs() > 0.001 {
                        let mut new_vp = self.viewport;
                        new_vp.zoom_at(cursor_pos, zoom_factor);
                        return Some(
                            Action::publish((self.on_viewport_changed)(new_vp)).and_capture(),
                        );
                    }
                } else if state.modifiers.shift() {
                    let shift_dx = if dy.abs() > dx.abs() { dy } else { dx };
                    let mut new_vp = self.viewport;
                    new_vp.translate(Vector::new(shift_dx, 0.0));
                    return Some(Action::publish((self.on_viewport_changed)(new_vp)).and_capture());
                } else {
                    let mut new_vp = self.viewport;
                    new_vp.translate(Vector::new(dx, dy));
                    return Some(Action::publish((self.on_viewport_changed)(new_vp)).and_capture());
                }
                None
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                state.panning_start = Some((cursor_pos, self.viewport.pan()));
                Some(Action::capture())
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if self.is_space_pressed {
                    state.panning_start = Some((cursor_pos, self.viewport.pan()));
                    state.is_panning_space = true;
                    return Some(Action::capture());
                }

                state.panning_start = None;
                state.is_panning_space = false;

                // 0. Tjek om der klikkes på det svævende ergonomi-panel eller miniaturekort (Minimap)
                let panel_rect = Self::floating_panel_rect(bounds);
                if panel_rect.contains(cursor_pos) {
                    let minimap_rect = Self::minimap_rect(panel_rect);
                    let zoom_in_rect = Self::zoom_in_button_rect(panel_rect);
                    let zoom_out_rect = Self::zoom_out_button_rect(panel_rect);
                    let fit_rect = Self::fit_view_button_rect(panel_rect);
                    let zoom_label_rect = Self::zoom_label_rect(panel_rect);

                    if zoom_in_rect.contains(cursor_pos) {
                        let mut new_vp = self.viewport;
                        new_vp.zoom_at(Point::new(bounds.width / 2.0, bounds.height / 2.0), 1.15);
                        return Some(
                            Action::publish((self.on_viewport_changed)(new_vp)).and_capture(),
                        );
                    }
                    if zoom_out_rect.contains(cursor_pos) {
                        let mut new_vp = self.viewport;
                        new_vp.zoom_at(
                            Point::new(bounds.width / 2.0, bounds.height / 2.0),
                            1.0 / 1.15,
                        );
                        return Some(
                            Action::publish((self.on_viewport_changed)(new_vp)).and_capture(),
                        );
                    }
                    if fit_rect.contains(cursor_pos) || zoom_label_rect.contains(cursor_pos) {
                        let new_vp = compute_fit_to_view(self.nodes, bounds.size());
                        return Some(
                            Action::publish((self.on_viewport_changed)(new_vp)).and_capture(),
                        );
                    }
                    if minimap_rect.contains(cursor_pos) {
                        state.is_panning_minimap = true;
                        let transform = MinimapTransform::compute(
                            self.nodes,
                            &self.viewport,
                            bounds.size(),
                            minimap_rect,
                        );
                        let world_pt = transform.minimap_to_world(cursor_pos);
                        let mut new_vp = self.viewport;
                        let new_pan = Vector::new(
                            bounds.width / 2.0 - world_pt.x * self.viewport.zoom(),
                            bounds.height / 2.0 - world_pt.y * self.viewport.zoom(),
                        );
                        new_vp.set_pan(new_pan);
                        return Some(
                            Action::publish((self.on_viewport_changed)(new_vp)).and_capture(),
                        );
                    }
                    // Klik på panelbaggrund absorberes så diagramnoder bagved ikke påvirkes
                    return Some(Action::capture());
                }

                let world_pos = self.viewport.to_world(cursor_pos);

                // 1. Tjek om der klikkes på forbindelseshåndtaget (connect handle) på den valgte node
                if let Some(sel_id) = self.selected_node_id {
                    if let Some(node) = self.nodes.iter().find(|n| n.id() == sel_id) {
                        let (nx, ny) = node.position();
                        let (nw, nh) = node.size();
                        let handle_center = Point::new(nx + nw, ny + nh / 2.0);
                        let dist =
                            (world_pos.x - handle_center.x).hypot(world_pos.y - handle_center.y);
                        if dist <= 14.0 {
                            state.connecting_from = Some(sel_id);
                            state.connecting_cursor = Some(world_pos);
                            state.hovered_target_node = None;
                            return Some(Action::request_redraw().and_capture());
                        }
                    }
                }

                let is_ctrl = state.modifiers.control() || state.modifiers.command();
                state.press_screen_pos = Some(cursor_pos);

                let target = self
                    .nodes
                    .iter()
                    .rev()
                    .find(|node| node.contains(world_pos.x, world_pos.y))
                    .map(|node| ClickTarget::Node(node.id()))
                    .unwrap_or(ClickTarget::Empty);

                if is_ctrl {
                    state.last_click = None;
                    state.pending_double = None;
                } else {
                    let now = Instant::now();
                    let is_candidate = if let Some(last) = state.last_click {
                        let dist =
                            (cursor_pos.x - last.position.x).hypot(cursor_pos.y - last.position.y);
                        let elapsed = now.duration_since(last.time).as_millis();
                        dist <= 20.0 && elapsed <= 500 && last.target == target
                    } else {
                        false
                    };

                    if is_candidate {
                        let first_world_pos = state
                            .last_click
                            .map(|l| l.world_position)
                            .unwrap_or(world_pos);
                        state.pending_double = Some((target, first_world_pos));
                    } else {
                        state.pending_double = None;
                    }
                }

                // 2. Klik på en node
                for node in self.nodes.iter().rev() {
                    if node.contains(world_pos.x, world_pos.y) {
                        let is_ctrl = state.modifiers.control() || state.modifiers.command();
                        if is_ctrl {
                            if state.selected_node_ids.contains(&node.id()) {
                                state.selected_node_ids.remove(&node.id());
                            } else {
                                state.selected_node_ids.insert(node.id());
                            }
                            let new_selection = state.selected_node_ids.clone();
                            if let Some(ref on_selection_changed) = self.on_selection_changed {
                                return Some(
                                    Action::publish((on_selection_changed)(new_selection))
                                        .and_capture(),
                                );
                            } else {
                                let first = new_selection.iter().next().copied();
                                return Some(
                                    Action::publish((self.on_node_selected)(first)).and_capture(),
                                );
                            }
                        } else {
                            let is_already_selected = state.selected_node_ids.contains(&node.id())
                                || self.is_node_selected(node.id());
                            let drag_ids: Vec<NodeId> =
                                if is_already_selected && state.selected_node_ids.len() > 1 {
                                    state.selected_node_ids.iter().copied().collect()
                                } else {
                                    vec![node.id()]
                                };

                            let initial_positions: Vec<(NodeId, (f32, f32))> = self
                                .nodes
                                .iter()
                                .filter(|n| drag_ids.contains(&n.id()))
                                .map(|n| (n.id(), n.position()))
                                .collect();

                            let (nx, ny) = node.position();
                            let offset = Vector::new(world_pos.x - nx, world_pos.y - ny);
                            state.dragging_node = Some((node.id(), offset));
                            state.bulk_drag = Some(BulkDragState {
                                leader_id: node.id(),
                                start_world_pos: world_pos,
                                initial_positions,
                                last_delta: None,
                                start_time: std::time::Instant::now(),
                            });
                            crate::features::diagnostics::log_info(
                                "canvas",
                                &format!(
                                    "Startede drag med {} noder (leader: {})",
                                    drag_ids.len(),
                                    node.id()
                                ),
                            );

                            if !is_already_selected {
                                state.selected_node_ids = [node.id()].into_iter().collect();
                                let new_selection = state.selected_node_ids.clone();
                                if let Some(ref on_selection_changed) = self.on_selection_changed {
                                    return Some(
                                        Action::publish((on_selection_changed)(new_selection))
                                            .and_capture(),
                                    );
                                } else {
                                    return Some(
                                        Action::publish((self.on_node_selected)(Some(node.id())))
                                            .and_capture(),
                                    );
                                }
                            } else {
                                return Some(Action::capture());
                            }
                        }
                    }
                }

                // 3. Klik på en relation (kant eller label)
                let diagram_nodes: Vec<DiagramNode> =
                    self.nodes.iter().map(|n| n.to_diagram_node()).collect();
                let diagram_edges: Vec<DiagramEdge> =
                    self.edges.iter().map(|e| e.to_diagram_edge()).collect();
                let routed_edges = EdgeRouter::route_edges(&diagram_nodes, &diagram_edges);

                for routed in routed_edges.iter().rev() {
                    let hit_segment = routed
                        .points
                        .windows(2)
                        .any(|w| distance_to_segment(world_pos, w[0], w[1]) <= 7.0);

                    let hit_label =
                        if let (Some(label), Some(pos)) = (&routed.label, routed.label_pos) {
                            let approx_w = label.trim().len() as f32 * 6.8 + 16.0;
                            let min_x = pos.x - approx_w / 2.0;
                            let max_x = pos.x + approx_w / 2.0;
                            let min_y = pos.y - 10.0;
                            let max_y = pos.y + 10.0;
                            world_pos.x >= min_x
                                && world_pos.x <= max_x
                                && world_pos.y >= min_y
                                && world_pos.y <= max_y
                        } else {
                            false
                        };

                    if hit_segment || hit_label {
                        if let Some(ref on_edge_selected) = self.on_edge_selected {
                            return Some(
                                Action::publish((on_edge_selected)(Some((routed.from, routed.to))))
                                    .and_capture(),
                            );
                        }
                    }
                }

                // 4. Klik på tomt lærred: fravælg kant og start potentiel marquee drag-selection
                if let Some(ref on_edge_selected) = self.on_edge_selected {
                    let _ = (on_edge_selected)(None);
                }
                state.marquee = Some(MarqueeState {
                    start: world_pos,
                    current: world_pos,
                });
                Some(Action::capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(press_pos) = state.press_screen_pos {
                    let dist = (cursor_pos.x - press_pos.x).hypot(cursor_pos.y - press_pos.y);
                    if dist >= CLICK_SLOP_PX {
                        state.pending_double = None;
                        state.last_click = None;
                    }
                }

                if state.is_panning_minimap {
                    let panel_rect = Self::floating_panel_rect(bounds);
                    let minimap_rect = Self::minimap_rect(panel_rect);
                    let transform = MinimapTransform::compute(
                        self.nodes,
                        &self.viewport,
                        bounds.size(),
                        minimap_rect,
                    );
                    let clamped_pos = Point::new(
                        cursor_pos
                            .x
                            .clamp(minimap_rect.x, minimap_rect.x + minimap_rect.width),
                        cursor_pos
                            .y
                            .clamp(minimap_rect.y, minimap_rect.y + minimap_rect.height),
                    );
                    let world_pt = transform.minimap_to_world(clamped_pos);
                    let mut new_vp = self.viewport;
                    let new_pan = Vector::new(
                        bounds.width / 2.0 - world_pt.x * self.viewport.zoom(),
                        bounds.height / 2.0 - world_pt.y * self.viewport.zoom(),
                    );
                    new_vp.set_pan(new_pan);
                    return Some(Action::publish((self.on_viewport_changed)(new_vp)).and_capture());
                }

                if let Some((start_pos, initial_pan)) = state.panning_start {
                    let total_delta = cursor_pos - start_pos;
                    let mut new_vp = self.viewport;
                    new_vp.set_pan(initial_pan + total_delta);
                    return Some(Action::publish((self.on_viewport_changed)(new_vp)).and_capture());
                }

                if let Some(source_id) = state.connecting_from {
                    let world_pos = self.viewport.to_world(cursor_pos);
                    state.connecting_cursor = Some(world_pos);
                    state.hovered_target_node = self
                        .nodes
                        .iter()
                        .rev()
                        .find(|n| n.id() != source_id && n.contains(world_pos.x, world_pos.y))
                        .map(|n| n.id());
                    return Some(Action::request_redraw().and_capture());
                }

                if let Some(ref mut marquee) = state.marquee {
                    let world_pos = self.viewport.to_world(cursor_pos);
                    marquee.current = world_pos;
                    return Some(Action::request_redraw().and_capture());
                }

                if let Some(ref mut bulk) = state.bulk_drag {
                    let screen_dist = state
                        .press_screen_pos
                        .map(|p| (cursor_pos.x - p.x).hypot(cursor_pos.y - p.y))
                        .unwrap_or(CLICK_SLOP_PX);
                    if screen_dist < CLICK_SLOP_PX {
                        return Some(Action::capture());
                    }

                    let world_pos = self.viewport.to_world(cursor_pos);
                    let raw_dx = world_pos.x - bulk.start_world_pos.x;
                    let raw_dy = world_pos.y - bulk.start_world_pos.y;

                    let (delta_x, delta_y) = if let Some(&(_, (lx, ly))) = bulk
                        .initial_positions
                        .iter()
                        .find(|(id, _)| *id == bulk.leader_id)
                    {
                        let target_x = lx + raw_dx;
                        let target_y = ly + raw_dy;
                        let snapped_x = (target_x / GRID_SIZE).round() * GRID_SIZE;
                        let snapped_y = (target_y / GRID_SIZE).round() * GRID_SIZE;
                        (snapped_x - lx, snapped_y - ly)
                    } else {
                        (raw_dx, raw_dy)
                    };

                    // AC2: Redundant Motion Delta Filtering
                    if bulk.last_delta == Some((delta_x, delta_y)) {
                        return Some(Action::capture());
                    }
                    bulk.last_delta = Some((delta_x, delta_y));

                    let updates: Vec<(NodeId, f32, f32)> = bulk
                        .initial_positions
                        .iter()
                        .map(|&(id, (ix, iy))| (id, ix + delta_x, iy + delta_y))
                        .collect();

                    if let Some(ref on_nodes_moved) = self.on_nodes_moved {
                        return Some(Action::publish((on_nodes_moved)(updates)).and_capture());
                    } else if let Some(&(lid, lx, ly)) =
                        updates.iter().find(|(id, _, _)| *id == bulk.leader_id)
                    {
                        return Some(
                            Action::publish((self.on_node_moved)(lid, lx, ly)).and_capture(),
                        );
                    }
                    return Some(Action::capture());
                } else if let Some((id, offset)) = state.dragging_node {
                    let screen_dist = state
                        .press_screen_pos
                        .map(|p| (cursor_pos.x - p.x).hypot(cursor_pos.y - p.y))
                        .unwrap_or(CLICK_SLOP_PX);
                    if screen_dist < CLICK_SLOP_PX {
                        return Some(Action::capture());
                    }

                    let world_pos = self.viewport.to_world(cursor_pos);
                    let raw_world_x = world_pos.x - offset.x;
                    let raw_world_y = world_pos.y - offset.y;

                    let new_x = (raw_world_x / GRID_SIZE).round() * GRID_SIZE;
                    let new_y = (raw_world_y / GRID_SIZE).round() * GRID_SIZE;

                    if let Some(node) = self.nodes.iter().find(|n| n.id() == id) {
                        let (cx, cy) = node.position();
                        if (cx - new_x).abs() < f32::EPSILON && (cy - new_y).abs() < f32::EPSILON {
                            return Some(Action::capture());
                        }
                    }

                    return Some(
                        Action::publish((self.on_node_moved)(id, new_x, new_y)).and_capture(),
                    );
                }

                None
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
                state.panning_start = None;
                Some(Action::capture())
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.is_panning_minimap {
                    state.is_panning_minimap = false;
                    return Some(Action::capture());
                }
                state.panning_start = None;
                let was_panning_space = state.is_panning_space;
                state.is_panning_space = false;
                if was_panning_space {
                    return Some(Action::capture());
                }

                if let Some(source_id) = state.connecting_from.take() {
                    state.connecting_cursor = None;
                    let target = state.hovered_target_node.take();
                    if let Some(target_id) = target {
                        if target_id != source_id {
                            if let Some(ref on_edge_created) = self.on_edge_created {
                                return Some(
                                    Action::publish((on_edge_created)(source_id, target_id))
                                        .and_capture(),
                                );
                            }
                        }
                    }
                    return Some(Action::request_redraw().and_capture());
                }

                let press_pos = state.press_screen_pos.take();
                let move_dist = press_pos
                    .map(|p| (cursor_pos.x - p.x).hypot(cursor_pos.y - p.y))
                    .unwrap_or(0.0);

                if let Some((target, first_world_pos)) = state.pending_double.take() {
                    if move_dist < CLICK_SLOP_PX {
                        state.bulk_drag = None;
                        state.dragging_node = None;
                        state.marquee = None;
                        state.last_click = None;
                        match target {
                            ClickTarget::Node(id) => {
                                return Some(
                                    Action::publish((self.on_node_double_clicked)(id))
                                        .and_capture(),
                                );
                            }
                            ClickTarget::Empty => {
                                return Some(
                                    Action::publish((self.on_canvas_double_clicked)(
                                        first_world_pos.x,
                                        first_world_pos.y,
                                    ))
                                    .and_capture(),
                                );
                            }
                        }
                    }
                }

                if let Some(marquee) = state.marquee.take() {
                    let is_drag = move_dist >= CLICK_SLOP_PX;
                    if !is_drag {
                        // Enkeltklik på tomt lærred (ikke drag)
                        if !state.modifiers.control() && !state.modifiers.command() {
                            state.last_click = Some(ClickRecord {
                                position: cursor_pos,
                                world_position: self.viewport.to_world(cursor_pos),
                                time: Instant::now(),
                                target: ClickTarget::Empty,
                            });
                            state.selected_node_ids.clear();
                            if let Some(ref on_edge_selected) = self.on_edge_selected {
                                let _ = (on_edge_selected)(None);
                            }
                            if let Some(ref on_selection_changed) = self.on_selection_changed {
                                return Some(
                                    Action::publish((on_selection_changed)(
                                        std::collections::HashSet::new(),
                                    ))
                                    .and_capture(),
                                );
                            } else {
                                return Some(
                                    Action::publish((self.on_node_selected)(None)).and_capture(),
                                );
                            }
                        }
                    } else {
                        // Rektangulær drag-select (marquee)
                        state.last_click = None;
                        let min_x = marquee.start.x.min(marquee.current.x);
                        let max_x = marquee.start.x.max(marquee.current.x);
                        let min_y = marquee.start.y.min(marquee.current.y);
                        let max_y = marquee.start.y.max(marquee.current.y);

                        let selected_in_box: std::collections::HashSet<NodeId> = self
                            .nodes
                            .iter()
                            .filter(|n| {
                                let (nx, ny) = n.position();
                                let (nw, nh) = n.size();
                                let node_max_x = nx + nw;
                                let node_max_y = ny + nh;
                                // Kun noder der er fuldt omsluttet af markeringsrammen vælges
                                nx >= min_x
                                    && node_max_x <= max_x
                                    && ny >= min_y
                                    && node_max_y <= max_y
                            })
                            .map(|n| n.id())
                            .collect();

                        let new_selection =
                            if state.modifiers.control() || state.modifiers.command() {
                                let mut combined = state.selected_node_ids.clone();
                                combined.extend(selected_in_box);
                                combined
                            } else {
                                selected_in_box
                            };
                        state.selected_node_ids = new_selection.clone();

                        if let Some(ref on_selection_changed) = self.on_selection_changed {
                            return Some(
                                Action::publish((on_selection_changed)(new_selection))
                                    .and_capture(),
                            );
                        } else {
                            let first = new_selection.iter().next().copied();
                            return Some(
                                Action::publish((self.on_node_selected)(first)).and_capture(),
                            );
                        }
                    }
                    return Some(Action::request_redraw().and_capture());
                }

                if let Some(bulk) = state.bulk_drag.take() {
                    state.dragging_node = None;
                    let is_drag = move_dist >= CLICK_SLOP_PX;
                    if is_drag {
                        state.last_click = None;
                        crate::features::diagnostics::log_info(
                            "canvas",
                            &format!(
                                "Drag fuldført for {} noder (skærm-distance: {:.1}px, varighed: {:?})",
                                bulk.initial_positions.len(),
                                move_dist,
                                bulk.start_time.elapsed()
                            ),
                        );
                        if let Some(ref on_drag_released) = self.on_drag_released {
                            return Some(Action::publish((on_drag_released)()).and_capture());
                        }
                    } else {
                        // Enkeltklik på en node uden bevægelse
                        if !state.modifiers.control() && !state.modifiers.command() {
                            state.last_click = Some(ClickRecord {
                                position: cursor_pos,
                                world_position: self.viewport.to_world(cursor_pos),
                                time: Instant::now(),
                                target: ClickTarget::Node(bulk.leader_id),
                            });
                        }
                        if state.selected_node_ids.len() > 1
                            && !state.modifiers.control()
                            && !state.modifiers.command()
                        {
                            state.selected_node_ids = [bulk.leader_id].into_iter().collect();
                            let new_selection = state.selected_node_ids.clone();
                            if let Some(ref on_selection_changed) = self.on_selection_changed {
                                return Some(
                                    Action::publish((on_selection_changed)(new_selection))
                                        .and_capture(),
                                );
                            } else {
                                return Some(
                                    Action::publish((self.on_node_selected)(Some(bulk.leader_id)))
                                        .and_capture(),
                                );
                            }
                        }
                    }
                    return Some(Action::capture());
                }

                if state.dragging_node.is_some() {
                    state.dragging_node = None;
                    let is_drag = move_dist >= CLICK_SLOP_PX;
                    if is_drag {
                        if let Some(ref on_drag_released) = self.on_drag_released {
                            return Some(Action::publish((on_drag_released)()).and_capture());
                        }
                    }
                    return Some(Action::capture());
                }

                if self.is_palette_dragging {
                    if let Some(ref on_canvas_drop) = self.on_canvas_drop {
                        let world_pos = self.viewport.to_world(cursor_pos);
                        return Some(
                            Action::publish((on_canvas_drop)(world_pos.x, world_pos.y))
                                .and_capture(),
                        );
                    }
                }

                None
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Baggrund
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            ColorTokens::CANVAS_BACKGROUND.into_color(),
        );

        // 1. Baggrundsgitter (dækker hele vinduet dynamisk ved resize og pan)
        let step = GRID_SIZE * self.viewport.zoom();
        let ox = self.viewport.pan().x.rem_euclid(step);
        let oy = self.viewport.pan().y.rem_euclid(step);

        let dot_color = if self.viewport.zoom() < 0.5 {
            ColorTokens::SLATE_400.with_alpha(0.25).into_color()
        } else {
            ColorTokens::SLATE_400.with_alpha(0.45).into_color()
        };

        let mut grid_builder = iced::widget::canvas::path::Builder::new();
        let mut gx = ox;
        while gx <= bounds.width {
            let mut gy = oy;
            while gy <= bounds.height {
                grid_builder.rectangle(Point::new(gx - 1.0, gy - 1.0), Size::new(2.0, 2.0));
                gy += step;
            }
            gx += step;
        }
        let grid_path = grid_builder.build();
        frame.fill(&grid_path, dot_color);

        // 2. Transformer resten af tegningen via frame transformation
        frame.translate(self.viewport.pan());
        frame.scale(self.viewport.zoom());

        // 2a. Svævende snap-forhåndsvisning ved træk fra palet
        if self.is_palette_dragging {
            if let Some(cursor_pos) = cursor.position_in(bounds) {
                let world_pos = self.viewport.to_world(cursor_pos);
                let snapped_x = (world_pos.x / GRID_SIZE).round() * GRID_SIZE;
                let snapped_y = (world_pos.y / GRID_SIZE).round() * GRID_SIZE;
                let preview_rect = Path::rounded_rectangle(
                    Point::new(snapped_x, snapped_y),
                    Size::new(140.0, 60.0),
                    8.0.into(),
                );
                frame.fill(&preview_rect, ColorTokens::SELECTION_FILL.into_color());
                frame.stroke(
                    &preview_rect,
                    Stroke::default()
                        .with_color(ThemeColors::PRIMARY)
                        .with_width(1.5),
                );
            }
        }

        // 3. Deterministisk ortogonal edge routing
        let diagram_nodes: Vec<DiagramNode> =
            self.nodes.iter().map(|n| n.to_diagram_node()).collect();
        let diagram_edges: Vec<DiagramEdge> =
            self.edges.iter().map(|e| e.to_diagram_edge()).collect();

        let routed_edges = EdgeRouter::route_edges(&diagram_nodes, &diagram_edges);

        // A. Tegn ortogonale linjeforløb med krydsningsbroer (line jumps)
        for routed in &routed_edges {
            if routed.points.len() < 2 {
                continue;
            }

            let is_selected_edge = self.selected_edge == Some((routed.from, routed.to))
                || self.selected_edge == Some((routed.to, routed.from));
            let edge_color = if is_selected_edge {
                ThemeColors::PRIMARY
            } else {
                ColorTokens::SLATE_600.into_color()
            };
            let edge_width = if is_selected_edge { 2.8 } else { 1.5 };

            let path = Path::new(|b| {
                b.move_to(routed.points[0]);

                for window in routed.points.windows(2) {
                    let (p1, p2) = (window[0], window[1]);
                    let is_h = (p1.y - p2.y).abs() < 0.001;

                    if is_h && !routed.bridges.is_empty() {
                        let min_x = p1.x.min(p2.x);
                        let max_x = p1.x.max(p2.x);
                        let mut seg_bridges: Vec<_> = routed
                            .bridges
                            .iter()
                            .filter(|br| {
                                br.is_horizontal
                                    && (br.center.y - p1.y).abs() < 0.001
                                    && br.center.x > min_x + 2.0
                                    && br.center.x < max_x - 2.0
                            })
                            .collect();

                        if p1.x < p2.x {
                            seg_bridges
                                .sort_by(|a, b| a.center.x.partial_cmp(&b.center.x).unwrap());
                        } else {
                            seg_bridges
                                .sort_by(|a, b| b.center.x.partial_cmp(&a.center.x).unwrap());
                        }

                        let r = 5.0;
                        for bridge in seg_bridges {
                            let bx = bridge.center.x;
                            let by = bridge.center.y;
                            let approach_x = if p1.x < p2.x { bx - r } else { bx + r };
                            let exit_x = if p1.x < p2.x { bx + r } else { bx - r };

                            b.line_to(Point::new(approach_x, by));
                            b.bezier_curve_to(
                                Point::new(approach_x, by - 6.0),
                                Point::new(exit_x, by - 6.0),
                                Point::new(exit_x, by),
                            );
                        }
                    }

                    b.line_to(p2);
                }
            });

            let stroke = if routed.kind == RelationKind::Dependency {
                Stroke {
                    style: iced::widget::canvas::stroke::Style::Solid(edge_color),
                    width: edge_width,
                    line_dash: iced::widget::canvas::stroke::LineDash {
                        segments: &[6.0, 4.0],
                        offset: 0,
                    },
                    ..Default::default()
                }
            } else {
                Stroke::default()
                    .with_color(edge_color)
                    .with_width(edge_width)
            };
            frame.stroke(&path, stroke);
        }

        // 4. Tegn noder via pluggable node-rendering closure
        for node in self.nodes {
            let is_selected = self.is_node_selected(node.id());
            (self.render_node)(&mut frame, node, is_selected, self.viewport);

            // Highlight målnode under drag-to-connect
            if state.hovered_target_node == Some(node.id()) {
                let (nx, ny) = node.position();
                let (nw, nh) = node.size();
                let target_glow = Path::rounded_rectangle(
                    Point::new(nx - 3.0, ny - 3.0),
                    Size::new(nw + 6.0, nh + 6.0),
                    10.0.into(),
                );
                frame.stroke(
                    &target_glow,
                    Stroke::default()
                        .with_color(ThemeColors::PRIMARY)
                        .with_width(3.0),
                );
            }

            // Forbindelseshåndtag (connect handle) på primær valgt node
            if is_selected
                && (self.selected_node_id == Some(node.id()) || self.selected_node_ids.len() <= 1)
            {
                let (nx, ny) = node.position();
                let (nw, nh) = node.size();
                let handle_center = Point::new(nx + nw, ny + nh / 2.0);
                let circle = Path::circle(handle_center, 8.0);
                frame.fill(&circle, Color::WHITE);
                frame.stroke(
                    &circle,
                    Stroke::default()
                        .with_color(ThemeColors::PRIMARY)
                        .with_width(2.0),
                );
                let inner_dot = Path::circle(handle_center, 3.5);
                frame.fill(&inner_dot, ThemeColors::PRIMARY);
            }
        }

        // 5. Elastik / Preview-linje under drag-to-connect
        if let (Some(source_id), Some(cursor_world)) =
            (state.connecting_from, state.connecting_cursor)
        {
            if let Some(src) = self.nodes.iter().find(|n| n.id() == source_id) {
                let (sx, sy) = src.position();
                let (sw, sh) = src.size();
                let start_pt = Point::new(sx + sw, sy + sh / 2.0);
                let preview = Path::line(start_pt, cursor_world);
                frame.stroke(
                    &preview,
                    Stroke::default()
                        .with_color(ThemeColors::PRIMARY)
                        .with_width(2.5),
                );
                let end_circle = Path::circle(cursor_world, 5.0);
                frame.fill(&end_circle, ThemeColors::PRIMARY);
            }
        }

        // 6. Tegn pilehoveder og labels ovenpå noder for optimal synlighed
        for routed in &routed_edges {
            let is_selected_edge = self.selected_edge == Some((routed.from, routed.to))
                || self.selected_edge == Some((routed.to, routed.from));
            let arrow_stroke_color = if is_selected_edge {
                ThemeColors::PRIMARY
            } else {
                ColorTokens::SLATE_800.into_color()
            };
            let arrow_stroke_width = if is_selected_edge { 2.2 } else { 1.5 };

            // 6a. Kompositions-diamant ved kilde-noden (solid sort diamant)
            if let Some(ref diamond) = routed.source_diamond {
                let diamond_path = Path::new(|b| {
                    b.move_to(diamond.tip);
                    b.line_to(diamond.left);
                    b.line_to(diamond.back);
                    b.line_to(diamond.right);
                    b.close();
                });
                frame.fill(&diamond_path, arrow_stroke_color);
                frame.stroke(
                    &diamond_path,
                    Stroke::default()
                        .with_color(arrow_stroke_color)
                        .with_width(arrow_stroke_width),
                );
            }

            // 6b. Pilehoved ved mål-noden
            if let Some(ref arrow) = routed.arrow_head {
                if routed.kind == RelationKind::Generalization {
                    let triangle = Path::new(|b| {
                        b.move_to(arrow.tip);
                        b.line_to(arrow.left);
                        b.line_to(arrow.right);
                        b.close();
                    });
                    frame.fill(&triangle, Color::WHITE);
                    frame.stroke(
                        &triangle,
                        Stroke::default()
                            .with_color(arrow_stroke_color)
                            .with_width(arrow_stroke_width),
                    );
                } else if routed.kind == RelationKind::Dependency {
                    let open_arrow = Path::new(|b| {
                        b.move_to(arrow.left);
                        b.line_to(arrow.tip);
                        b.line_to(arrow.right);
                    });
                    frame.stroke(
                        &open_arrow,
                        Stroke::default()
                            .with_color(arrow_stroke_color)
                            .with_width(arrow_stroke_width + 0.5),
                    );
                }
            }

            // 6c. Halv pil for rettet association ved mål-noden
            if let Some(ref half_arrow) = routed.half_arrow {
                let barb_line = Path::line(half_arrow.tip, half_arrow.barb);
                frame.stroke(
                    &barb_line,
                    Stroke::default()
                        .with_color(arrow_stroke_color)
                        .with_width(arrow_stroke_width + 0.5),
                );
            }

            if let (Some(label), Some(pos)) = (&routed.label, routed.label_pos) {
                if !label.trim().is_empty() {
                    let label_text = label.trim();
                    let approx_w = label_text.len() as f32 * 6.8 + 12.0;
                    let pill = Path::rounded_rectangle(
                        Point::new(pos.x - approx_w / 2.0, pos.y - 8.0),
                        Size::new(approx_w, 16.0),
                        4.0.into(),
                    );
                    frame.fill(&pill, ColorTokens::SURFACE.with_alpha(0.90).into_color());
                    if is_selected_edge {
                        frame.stroke(
                            &pill,
                            Stroke::default()
                                .with_color(ThemeColors::PRIMARY)
                                .with_width(1.5),
                        );
                    }

                    frame.fill_text(Text {
                        content: label_text.to_string(),
                        position: pos,
                        color: if is_selected_edge {
                            ThemeColors::PRIMARY
                        } else {
                            ThemeColors::SLATE_800
                        },
                        size: 11.0.into(),
                        align_x: alignment::Horizontal::Center.into(),
                        align_y: alignment::Vertical::Center,
                        ..Default::default()
                    });
                }
            }

            // 6d. Multipliciteter ved kilde- og målporte (UML standard §6)
            if let Some(edge) = self
                .edges
                .iter()
                .find(|e| e.from() == routed.from && e.to() == routed.to && e.kind() == routed.kind)
            {
                if let Some(src_mult) = edge.source_multiplicity() {
                    let p_src = routed.points[0];
                    let pos = port_multiplicity_pos(p_src, routed.from_side);
                    render_multiplicity_label(&mut frame, &src_mult, pos, is_selected_edge);
                }

                if let Some(tgt_mult) = edge.target_multiplicity() {
                    let p_tgt = *routed.points.last().unwrap_or(&routed.points[0]);
                    let pos = port_multiplicity_pos(p_tgt, routed.to_side);
                    render_multiplicity_label(&mut frame, &tgt_mult, pos, is_selected_edge);
                }
            }
        }

        // 6e. Rektangulær drag-select markeringsramme (Marquee) i verdenskoordinater
        if let Some(marquee) = state.marquee {
            let min_x = marquee.start.x.min(marquee.current.x);
            let max_x = marquee.start.x.max(marquee.current.x);
            let min_y = marquee.start.y.min(marquee.current.y);
            let max_y = marquee.start.y.max(marquee.current.y);
            let w = max_x - min_x;
            let h = max_y - min_y;
            if w > 0.0 && h > 0.0 {
                let rect = Path::rectangle(Point::new(min_x, min_y), Size::new(w, h));
                frame.fill(&rect, ColorTokens::SELECTION_FILL.into_color());
                frame.stroke(
                    &rect,
                    Stroke::default()
                        .with_color(ThemeColors::PRIMARY)
                        .with_width(1.5),
                );
            }
        }

        // 7. Svævende kontrolpanel og miniaturekort (Minimap) i skærmkoordinater
        let mut overlay_frame = Frame::new(renderer, bounds.size());
        let panel_rect = Self::floating_panel_rect(bounds);
        let minimap_rect = Self::minimap_rect(panel_rect);
        let zoom_in_rect = Self::zoom_in_button_rect(panel_rect);
        let zoom_out_rect = Self::zoom_out_button_rect(panel_rect);
        let fit_rect = Self::fit_view_button_rect(panel_rect);
        let zoom_label_rect = Self::zoom_label_rect(panel_rect);

        // A. Panelets baggrund og skygge (COSMIC semi-transparent glas-styling)
        let panel_shadow = Path::rounded_rectangle(
            Point::new(panel_rect.x, panel_rect.y + 3.0),
            panel_rect.size(),
            10.0.into(),
        );
        overlay_frame.fill(
            &panel_shadow,
            ColorTokens::SLATE_900.with_alpha(0.12).into_color(),
        );

        let panel_path = Path::rounded_rectangle(
            Point::new(panel_rect.x, panel_rect.y),
            panel_rect.size(),
            10.0.into(),
        );
        overlay_frame.fill(
            &panel_path,
            ColorTokens::SURFACE.with_alpha(0.92).into_color(),
        );
        overlay_frame.stroke(
            &panel_path,
            Stroke::default()
                .with_color(ThemeColors::SLATE_200)
                .with_width(1.0),
        );

        // B. Minimap baggrund
        let minimap_bg = Path::rounded_rectangle(
            Point::new(minimap_rect.x, minimap_rect.y),
            minimap_rect.size(),
            6.0.into(),
        );
        overlay_frame.fill(
            &minimap_bg,
            ColorTokens::CANVAS_BACKGROUND.with_alpha(0.95).into_color(),
        );
        overlay_frame.stroke(
            &minimap_bg,
            Stroke::default()
                .with_color(ColorTokens::SLATE_200.with_alpha(0.80).into_color())
                .with_width(0.8),
        );

        // C. Noder i minimappet
        let transform =
            MinimapTransform::compute(self.nodes, &self.viewport, bounds.size(), minimap_rect);
        for node in self.nodes {
            let (nx, ny) = node.position();
            let (nw, nh) = node.size();
            let top_left = transform.world_to_minimap(Point::new(nx, ny));
            let mini_w = (nw * transform.scale).max(3.0);
            let mini_h = (nh * transform.scale).max(3.0);

            let is_selected = self.is_node_selected(node.id());
            let node_rect =
                Path::rounded_rectangle(top_left, Size::new(mini_w, mini_h), 1.5.into());
            let node_color = if is_selected {
                ThemeColors::PRIMARY
            } else {
                ColorTokens::SLATE_400.into_color()
            };
            overlay_frame.fill(&node_rect, node_color);
        }

        // D. Viewport ramme i minimappet
        let vp_box = transform.viewport_rect(&self.viewport, bounds.size());
        let vp_path =
            Path::rounded_rectangle(Point::new(vp_box.x, vp_box.y), vp_box.size(), 2.0.into());
        overlay_frame.fill(&vp_path, ColorTokens::SELECTION_VIEWPORT.into_color());
        overlay_frame.stroke(
            &vp_path,
            Stroke::default()
                .with_color(ThemeColors::PRIMARY)
                .with_width(1.2),
        );

        // E. Knapper i værktøjslinjen (–, 100%, +, ⊡ Fit)
        let draw_btn = |f: &mut Frame, r: Rectangle, label: &str, fsize: f32, is_accent: bool| {
            let btn_path = Path::rounded_rectangle(Point::new(r.x, r.y), r.size(), 5.0.into());
            let bg = if is_accent {
                ColorTokens::SELECTION_FILL.into_color()
            } else {
                ColorTokens::SLATE_100.with_alpha(0.90).into_color()
            };
            f.fill(&btn_path, bg);
            f.stroke(
                &btn_path,
                Stroke::default()
                    .with_color(if is_accent {
                        ThemeColors::PRIMARY
                    } else {
                        ThemeColors::SLATE_200
                    })
                    .with_width(0.8),
            );
            f.fill_text(Text {
                content: label.to_string(),
                position: Point::new(r.x + r.width / 2.0, r.y + r.height / 2.0),
                color: if is_accent {
                    ThemeColors::PRIMARY
                } else {
                    ThemeColors::SLATE_800
                },
                size: fsize.into(),
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                ..Default::default()
            });
        };

        let zoom_pct = (self.viewport.zoom() * 100.0).round() as u32;
        draw_btn(&mut overlay_frame, zoom_out_rect, "–", 14.0, false);
        draw_btn(
            &mut overlay_frame,
            zoom_label_rect,
            &format!("{}%", zoom_pct),
            11.0,
            false,
        );
        draw_btn(&mut overlay_frame, zoom_in_rect, "+", 14.0, false);
        draw_btn(&mut overlay_frame, fit_rect, "⊡ Fit", 11.0, true);

        vec![frame.into_geometry(), overlay_frame.into_geometry()]
    }
}

/// Beregner placering af multiplicitetstekst ved port
fn port_multiplicity_pos(pt: Point, side: PortSide) -> Point {
    match side {
        PortSide::Right => Point::new(pt.x + 14.0, pt.y - 10.0),
        PortSide::Left => Point::new(pt.x - 14.0, pt.y - 10.0),
        PortSide::Top => Point::new(pt.x + 10.0, pt.y - 12.0),
        PortSide::Bottom => Point::new(pt.x + 10.0, pt.y + 12.0),
    }
}

/// Renderer multiplicitetstekst med baggrundspille
fn render_multiplicity_label(frame: &mut Frame, text: &str, pos: Point, is_selected: bool) {
    let approx_w = text.len() as f32 * 6.5 + 8.0;
    let pill = Path::rounded_rectangle(
        Point::new(pos.x - approx_w / 2.0, pos.y - 7.0),
        Size::new(approx_w, 14.0),
        3.0.into(),
    );
    frame.fill(&pill, ColorTokens::SURFACE.with_alpha(0.88).into_color());
    frame.fill_text(Text {
        content: text.to_string(),
        position: pos,
        color: if is_selected {
            ThemeColors::PRIMARY
        } else {
            ThemeColors::SLATE_700
        },
        size: 10.5.into(),
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center,
        ..Default::default()
    });
}

/// Standard rendering af FDA begrebsnode
pub fn render_concept_node(
    frame: &mut Frame,
    node: &DiagramNode,
    is_selected: bool,
    _viewport: CanvasViewport,
) {
    let top_left = Point::new(node.x(), node.y());
    let size = Size::new(node.width(), node.height());
    let radius = 8.0;

    let shadow_path =
        Path::rounded_rectangle(Point::new(node.x(), node.y() + 2.0), size, radius.into());
    frame.fill(&shadow_path, ColorTokens::SHADOW_LIGHT.into_color());

    let node_path = Path::rounded_rectangle(top_left, size, radius.into());

    let fill_color = if node.is_local() {
        ThemeColors::FDA_SAND
    } else {
        ThemeColors::FDA_BORROWED_BLUE
    };

    frame.fill(&node_path, fill_color);

    let (border_color, border_width) = if is_selected {
        (ThemeColors::PRIMARY, 2.5)
    } else if node.is_local() {
        (ThemeColors::FDA_SAND_BORDER, 1.2)
    } else {
        (ColorTokens::FDA_BORROWED_ACCENT.into_color(), 1.2)
    };

    frame.stroke(
        &node_path,
        Stroke::default()
            .with_color(border_color)
            .with_width(border_width),
    );

    let badge_text = concept_stereotype_text();

    frame.fill_text(Text {
        content: badge_text.to_string(),
        position: Point::new(node.x() + node.width() / 2.0, node.y() + 24.0),
        color: ThemeColors::SLATE_600,
        size: 10.5.into(),
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center,
        ..Default::default()
    });

    frame.fill_text(Text {
        content: node.label().to_string(),
        position: Point::new(node.x() + node.width() / 2.0, node.y() + 48.0),
        color: ThemeColors::SLATE_900,
        size: 14.0.into(),
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center,
        ..Default::default()
    });
}

/// Officiel FDA stereotype for begreber jf. Modelregel 03
pub fn concept_stereotype_text() -> &'static str {
    "«Concept»"
}

/// Officiel FDA stereotype for UML klasser jf. Modelregel 03
pub fn uml_class_stereotype_text(is_abstract: bool) -> String {
    if is_abstract {
        "«Concept» {abstract}".to_string()
    } else {
        "«Concept»".to_string()
    }
}

/// Officielt FDA UML keyword for enumerationer jf. Modelreglerne kapitel 5.2
pub fn enumeration_keyword_text() -> &'static str {
    "«enumeration»"
}

/// Officielt FDA UML keyword for strukturerede datatyper jf. Modelreglerne kapitel 5.5
pub fn datatype_keyword_text() -> &'static str {
    "«dataType»"
}

/// Rendering af UML Enumeration kasse (grøn jf. FDA Modelreglerne Kapitel 5.2 & 7.3)
pub fn render_uml_enumeration_node(
    frame: &mut Frame,
    node: &ClassDiagramNode,
    name: &str,
    values: &[String],
    is_selected: bool,
    _viewport: CanvasViewport,
) {
    let top_left = Point::new(node.x(), node.y());
    let size = Size::new(node.width(), node.height());
    let radius = 6.0;

    let shadow_path =
        Path::rounded_rectangle(Point::new(node.x(), node.y() + 2.0), size, radius.into());
    frame.fill(&shadow_path, ColorTokens::SHADOW_LIGHT.into_color());

    let node_path = Path::rounded_rectangle(top_left, size, radius.into());
    frame.fill(&node_path, ThemeColors::FDA_ENUM_GREEN);

    let (border_color, border_width) = if is_selected {
        (ThemeColors::PRIMARY, 2.5)
    } else {
        (ColorTokens::FDA_ENUM_ACCENT.into_color(), 1.2)
    };

    frame.stroke(
        &node_path,
        Stroke::default()
            .with_color(border_color)
            .with_width(border_width),
    );

    // Header: «enumeration» + Name
    let header_y = node.y() + 10.0;
    frame.fill_text(Text {
        content: enumeration_keyword_text().to_string(),
        position: Point::new(node.x() + node.width() / 2.0, header_y),
        color: ThemeColors::SLATE_600,
        size: 11.0.into(),
        align_x: iced::alignment::Horizontal::Center.into(),
        ..Default::default()
    });

    frame.fill_text(Text {
        content: name.to_string(),
        position: Point::new(node.x() + node.width() / 2.0, header_y + 16.0),
        color: ThemeColors::SLATE_900,
        size: 14.0.into(),
        align_x: iced::alignment::Horizontal::Center.into(),
        ..Default::default()
    });

    let divider_y = node.y() + 48.0;
    let divider = Path::line(
        Point::new(node.x(), divider_y),
        Point::new(node.x() + node.width(), divider_y),
    );
    frame.stroke(
        &divider,
        Stroke::default()
            .with_color(ColorTokens::SUCCESS_BORDER.into_color())
            .with_width(1.0),
    );

    // Values in lowerCamelCase
    let mut val_y = divider_y + 8.0;
    for val in values {
        frame.fill_text(Text {
            content: val.clone(),
            position: Point::new(node.x() + 12.0, val_y),
            color: ThemeColors::SLATE_800,
            size: 12.0.into(),
            ..Default::default()
        });
        val_y += 18.0;
    }
}

/// Rendering af UML Datatype kasse (gul jf. FDA Modelreglerne Kapitel 5.5 & 7.3)
pub fn render_uml_datatype_node(
    frame: &mut Frame,
    node: &ClassDiagramNode,
    name: &str,
    attributes: &[(String, String, String, bool)],
    is_selected: bool,
    _viewport: CanvasViewport,
) {
    let top_left = Point::new(node.x(), node.y());
    let size = Size::new(node.width(), node.height());
    let radius = 6.0;

    let shadow_path =
        Path::rounded_rectangle(Point::new(node.x(), node.y() + 2.0), size, radius.into());
    frame.fill(&shadow_path, ColorTokens::SHADOW_LIGHT.into_color());

    let node_path = Path::rounded_rectangle(top_left, size, radius.into());
    frame.fill(&node_path, ThemeColors::FDA_DATA_TYPE_YELLOW);

    let (border_color, border_width) = if is_selected {
        (ThemeColors::PRIMARY, 2.5)
    } else {
        (ColorTokens::FDA_SAND_ACCENT.into_color(), 1.2)
    };

    frame.stroke(
        &node_path,
        Stroke::default()
            .with_color(border_color)
            .with_width(border_width),
    );

    // Header: «dataType» + Name
    let header_y = node.y() + 10.0;
    frame.fill_text(Text {
        content: datatype_keyword_text().to_string(),
        position: Point::new(node.x() + node.width() / 2.0, header_y),
        color: ThemeColors::SLATE_600,
        size: 11.0.into(),
        align_x: iced::alignment::Horizontal::Center.into(),
        ..Default::default()
    });

    frame.fill_text(Text {
        content: name.to_string(),
        position: Point::new(node.x() + node.width() / 2.0, header_y + 16.0),
        color: ThemeColors::SLATE_900,
        size: 14.0.into(),
        align_x: iced::alignment::Horizontal::Center.into(),
        ..Default::default()
    });

    let divider_y = node.y() + 48.0;
    let divider = Path::line(
        Point::new(node.x(), divider_y),
        Point::new(node.x() + node.width(), divider_y),
    );
    frame.stroke(
        &divider,
        Stroke::default()
            .with_color(ColorTokens::WARNING_BORDER.into_color())
            .with_width(1.0),
    );

    // Attributes
    let mut attr_y = divider_y + 8.0;
    for (attr_name, attr_type, mult, _has_concept) in attributes {
        let text_content = format!("{}: {} [{}]", attr_name, attr_type, mult);
        frame.fill_text(Text {
            content: text_content,
            position: Point::new(node.x() + 12.0, attr_y),
            color: ThemeColors::SLATE_800,
            size: 12.0.into(),
            ..Default::default()
        });
        attr_y += 18.0;
    }
}

/// Afkorter en tekststreng pænt med ellipsis `...`, hvis den overstiger max_chars
pub fn truncate_with_ellipsis(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let prefix: String = s.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{}...", prefix)
    }
}

/// Standard rendering af UML 3-sektions kasse
#[allow(clippy::too_many_arguments)]
pub fn render_uml_class_node(
    frame: &mut Frame,
    node: &ClassDiagramNode,
    class_name: &str,
    attributes: &[(String, String, String, bool)],
    is_borrowed: bool,
    is_abstract: bool,
    is_selected: bool,
    _viewport: CanvasViewport,
) {
    let top_left = Point::new(node.x(), node.y());
    let size = Size::new(node.width(), node.height());
    let radius = 6.0;

    let shadow_path =
        Path::rounded_rectangle(Point::new(node.x(), node.y() + 2.0), size, radius.into());
    frame.fill(&shadow_path, ColorTokens::SHADOW_LIGHT.into_color());

    let node_path = Path::rounded_rectangle(top_left, size, radius.into());

    let fill_color = if is_borrowed {
        ThemeColors::FDA_BORROWED_BLUE
    } else {
        ThemeColors::FDA_SAND
    };

    frame.fill(&node_path, fill_color);

    let (border_color, border_width) = if is_selected {
        (ThemeColors::PRIMARY, 2.5)
    } else if is_borrowed {
        (ColorTokens::FDA_BORROWED_ACCENT.into_color(), 1.2)
    } else {
        (ThemeColors::FDA_SAND_BORDER, 1.2)
    };

    frame.stroke(
        &node_path,
        Stroke::default()
            .with_color(border_color)
            .with_width(border_width),
    );

    let stereotype_text = uml_class_stereotype_text(is_abstract);

    frame.fill_text(Text {
        content: stereotype_text,
        position: Point::new(node.x() + node.width() / 2.0, node.y() + 14.0),
        color: ThemeColors::SLATE_600,
        size: 10.5.into(),
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center,
        ..Default::default()
    });

    let name_font = if is_abstract {
        iced::Font {
            style: iced::font::Style::Italic,
            ..Default::default()
        }
    } else {
        iced::Font::DEFAULT
    };

    let display_class_name = truncate_with_ellipsis(class_name, 22);
    frame.fill_text(Text {
        content: display_class_name,
        position: Point::new(node.x() + node.width() / 2.0, node.y() + 32.0),
        color: ThemeColors::SLATE_900,
        size: 14.0.into(),
        font: name_font,
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center,
        ..Default::default()
    });

    let divider_y = node.y() + 48.0;
    let divider_path = Path::new(|b| {
        b.move_to(Point::new(node.x(), divider_y));
        b.line_to(Point::new(node.x() + node.width(), divider_y));
    });

    let divider_color = if is_borrowed {
        ColorTokens::INFO_BORDER.into_color()
    } else {
        ThemeColors::FDA_SAND_BORDER
    };

    frame.stroke(
        &divider_path,
        Stroke::default().with_color(divider_color).with_width(1.0),
    );

    let mut attr_y = divider_y + 14.0;
    if attributes.is_empty() {
        frame.fill_text(Text {
            content: "(ingen attributter)".to_string(),
            position: Point::new(node.x() + 14.0, attr_y),
            color: ThemeColors::SLATE_400,
            size: 11.0.into(),
            align_x: alignment::Horizontal::Left.into(),
            align_y: alignment::Vertical::Center,
            ..Default::default()
        });
    } else {
        for (attr_name, attr_type, attr_mult, has_concept) in attributes {
            let line_str = if *has_concept {
                format!("+ {} : {} [{}] 🔗", attr_name, attr_type, attr_mult)
            } else {
                format!("+ {} : {} [{}]", attr_name, attr_type, attr_mult)
            };
            let display_str = truncate_with_ellipsis(&line_str, 26);
            frame.fill_text(Text {
                content: display_str,
                position: Point::new(node.x() + 14.0, attr_y),
                color: ThemeColors::SLATE_800,
                size: 12.0.into(),
                align_x: alignment::Horizontal::Left.into(),
                align_y: alignment::Vertical::Center,
                ..Default::default()
            });
            attr_y += 20.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_viewport_math() {
        let mut vp = CanvasViewport::default();
        assert_eq!(vp.zoom(), 1.0);
        assert_eq!(vp.pan(), Vector::new(0.0, 0.0));

        let screen_pt = Point::new(100.0, 100.0);
        let world_pt = vp.to_world(screen_pt);
        assert_eq!(world_pt, Point::new(100.0, 100.0));

        vp.translate(Vector::new(50.0, 50.0));
        let world_pt2 = vp.to_world(screen_pt);
        assert_eq!(world_pt2, Point::new(50.0, 50.0));

        vp.set_zoom(2.0);
        assert_eq!(vp.zoom(), CanvasViewport::MAX_ZOOM);

        vp.set_zoom(0.1);
        assert_eq!(vp.zoom(), CanvasViewport::MIN_ZOOM);

        let snap_test = Point::new(23.4, 38.9);
        let snapped = CanvasViewport::snap_to_grid(snap_test, 20.0);
        assert_eq!(snapped, Point::new(20.0, 40.0));
    }

    #[test]
    fn test_pan_drag_cursor_tracking_does_not_drift() {
        let mut state = DiagramCanvasState::default();
        let initial_pan = Vector::new(10.0, 20.0);
        let click_pos = Point::new(100.0, 100.0);
        state.panning_start = Some((click_pos, initial_pan));

        // Simuler flere CursorMoved hændelser i samme frame
        let moves = [
            Point::new(120.0, 100.0),
            Point::new(150.0, 100.0),
            Point::new(200.0, 100.0),
        ];

        let mut last_pan = Vector::default();
        for current_cursor in moves {
            let (start_pos, init_pan) = state.panning_start.unwrap();
            let total_delta = current_cursor - start_pos;
            let current_pan = init_pan + total_delta;
            last_pan = current_pan;
        }

        // Slutpositionen for pan skal svare nøjagtigt til total cursor-bevægelse (+100px i X)
        assert_eq!(last_pan, Vector::new(110.0, 20.0));
    }

    #[test]
    fn test_node_selection_and_dragging_lifecycle() {
        use iced::mouse::Cursor;
        use std::sync::Arc;
        use uuid::Uuid;

        let node = DiagramNode::custom(
            Uuid::new_v4(),
            "TestNode".to_string(),
            100.0,
            100.0,
            180.0,
            80.0,
        );
        let nodes = vec![node];
        let edges: Vec<DiagramEdge> = vec![];

        let selected = Arc::new(std::sync::Mutex::new(None));
        let moved = Arc::new(std::sync::Mutex::new(None));

        let sel_clone = Arc::clone(&selected);
        let mov_clone = Arc::clone(&moved);

        let canvas = DiagramCanvas::new(
            &nodes,
            &edges,
            None,
            CanvasViewport::default(),
            false, // is_space_pressed = false
            |_, _, _, _| {},
            move |id| {
                *sel_clone.lock().unwrap() = id;
            },
            move |id, x, y| {
                *mov_clone.lock().unwrap() = Some((id, x, y));
            },
            |_, _| (),
            |_| (),
            |_| (),
        );

        let mut state = DiagramCanvasState::default();
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(1000.0, 1000.0));
        let cursor_on_node = Cursor::Available(Point::new(120.0, 120.0));

        // 1. Left click på node -> skal vælge noden og initialisere dragging_node
        let press_event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        let action = canvas.update(&mut state, &press_event, bounds, cursor_on_node);
        assert!(action.is_some());
        assert_eq!(*selected.lock().unwrap(), Some(nodes[0].id()));
        assert!(state.dragging_node.is_some());

        // 2. CursorMoved -> skal flytte noden
        let move_cursor = Cursor::Available(Point::new(160.0, 140.0));
        let move_event = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(160.0, 140.0),
        });
        let action = canvas.update(&mut state, &move_event, bounds, move_cursor);
        assert!(action.is_some());
        assert!(moved.lock().unwrap().is_some());

        // 3. ButtonReleased -> skal rydde dragging_node og panning_start
        let release_event = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
        let _ = canvas.update(&mut state, &release_event, bounds, move_cursor);
        assert!(state.dragging_node.is_none());
        assert!(state.panning_start.is_none());
    }

    #[test]
    fn test_task_067_diagram_canvas_motion_filtering_and_drag_release_callback() {
        use iced::mouse::Cursor;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use uuid::Uuid;

        let node = DiagramNode::custom(
            Uuid::new_v4(),
            "OptimizedNode".to_string(),
            100.0,
            100.0,
            180.0,
            80.0,
        );
        let nodes = vec![node];
        let edges: Vec<DiagramEdge> = vec![];

        let moves_count = Arc::new(AtomicUsize::new(0));
        let release_count = Arc::new(AtomicUsize::new(0));

        let m_clone = Arc::clone(&moves_count);
        let r_clone = Arc::clone(&release_count);

        let canvas = DiagramCanvas::new(
            &nodes,
            &edges,
            None,
            CanvasViewport::default(),
            false,
            |_, _, _, _| {},
            |_| (),
            |_, _, _| (),
            |_, _| (),
            |_| (),
            |_| (),
        )
        .on_nodes_moved(move |_updates| {
            m_clone.fetch_add(1, Ordering::SeqCst);
        })
        .on_drag_released(move || {
            r_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut state = DiagramCanvasState::default();
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(1000.0, 1000.0));
        let cursor_start = Cursor::Available(Point::new(120.0, 120.0));

        // 1. Klik for at starte drag
        let press = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        let _ = canvas.update(&mut state, &press, bounds, cursor_start);
        assert!(state.bulk_drag.is_some());
        assert_eq!(moves_count.load(Ordering::SeqCst), 0);

        // 2. Cursor flyttes med 20px (nyt snap step) -> skal udsende event
        let cursor_step1 = Cursor::Available(Point::new(140.0, 120.0));
        let move1 = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(140.0, 120.0),
        });
        let _ = canvas.update(&mut state, &move1, bounds, cursor_step1);
        assert_eq!(moves_count.load(Ordering::SeqCst), 1);

        // 3. Cursor flyttes med 2px (inden for samme snap-celle) -> skal filtreres (AC2)
        let cursor_subpixel = Cursor::Available(Point::new(142.0, 121.0));
        let move_subpixel = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(142.0, 121.0),
        });
        let _ = canvas.update(&mut state, &move_subpixel, bounds, cursor_subpixel);
        assert_eq!(
            moves_count.load(Ordering::SeqCst),
            1,
            "Redundant cursor motion må IKKE udsende ny besked"
        );

        // 4. Cursor flyttes med yderligere 20px -> skal udsende event
        let cursor_step2 = Cursor::Available(Point::new(160.0, 120.0));
        let move2 = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(160.0, 120.0),
        });
        let _ = canvas.update(&mut state, &move2, bounds, cursor_step2);
        assert_eq!(moves_count.load(Ordering::SeqCst), 2);

        // 5. ButtonReleased -> skal udløse on_drag_released (AC1)
        let release = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
        let _ = canvas.update(&mut state, &release, bounds, cursor_step2);
        assert_eq!(
            release_count.load(Ordering::SeqCst),
            1,
            "on_drag_released skal udløses ved drop"
        );
        assert!(state.bulk_drag.is_none());
    }

    #[test]
    fn test_drag_to_connect_lifecycle_and_edge_selection() {
        use iced::mouse::Cursor;
        use std::sync::Arc;
        use uuid::Uuid;

        let node1 = DiagramNode::custom(
            Uuid::new_v4(),
            "Node1".to_string(),
            100.0,
            100.0,
            180.0,
            80.0,
        );
        let node2 = DiagramNode::custom(
            Uuid::new_v4(),
            "Node2".to_string(),
            400.0,
            100.0,
            180.0,
            80.0,
        );
        let nodes = vec![node1.clone(), node2.clone()];
        let edge = DiagramEdge::with_label(
            node1.id(),
            node2.id(),
            RelationKind::Association,
            Some("omfatter".to_string()),
        );
        let edges = vec![edge];

        let created_edge = Arc::new(std::sync::Mutex::new(None));
        let selected_edge = Arc::new(std::sync::Mutex::new(None));

        let created_clone = Arc::clone(&created_edge);
        let selected_clone = Arc::clone(&selected_edge);

        let canvas = DiagramCanvas::new(
            &nodes,
            &edges,
            Some(node1.id()),
            CanvasViewport::default(),
            false,
            |_, _, _, _| {},
            |_| (),
            |_, _, _| (),
            |_, _| (),
            |_| (),
            |_| (),
        )
        .selected_edge(None)
        .on_edge_selected(move |e| {
            *selected_clone.lock().unwrap() = e;
        })
        .on_edge_created(move |from, to| {
            *created_clone.lock().unwrap() = Some((from, to));
        });

        let mut state = DiagramCanvasState::default();
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(1000.0, 1000.0));

        // 1. Klik på forbindelseshåndtaget for node1: placeret ved (nx + nw, ny + nh/2) = (280.0, 140.0)
        let handle_cursor = Cursor::Available(Point::new(280.0, 140.0));
        let press_event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        let action = canvas.update(&mut state, &press_event, bounds, handle_cursor);
        assert!(action.is_some());
        assert_eq!(state.connecting_from, Some(node1.id()));
        assert_eq!(state.connecting_cursor, Some(Point::new(280.0, 140.0)));

        // 2. CursorMoved over Node 2 (f.eks. ved (450.0, 120.0))
        let target_cursor = Cursor::Available(Point::new(450.0, 120.0));
        let move_event = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(450.0, 120.0),
        });
        let action = canvas.update(&mut state, &move_event, bounds, target_cursor);
        assert!(action.is_some());
        assert_eq!(state.hovered_target_node, Some(node2.id()));

        // 3. Drop: ButtonReleased over Node 2 -> skal udstede on_edge_created(node1, node2)
        let release_event = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
        let action = canvas.update(&mut state, &release_event, bounds, target_cursor);
        assert!(action.is_some());
        assert_eq!(
            *created_edge.lock().unwrap(),
            Some((node1.id(), node2.id()))
        );
        assert_eq!(state.connecting_from, None);
        assert_eq!(state.hovered_target_node, None);

        // 4. Klik på kanten/label for at vælge kanten
        // Mellem node1 højre kant (280, 140) og node2 venstre kant (400, 140) er der et vandret linjestykke ved y=140
        let edge_cursor = Cursor::Available(Point::new(340.0, 140.0));
        let press_event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        let action = canvas.update(&mut state, &press_event, bounds, edge_cursor);
        assert!(action.is_some());
        assert_eq!(
            *selected_edge.lock().unwrap(),
            Some((node1.id(), node2.id()))
        );

        // 5. Klik på tomt lærred (50, 50) -> skal rydde kantmarkering
        let empty_cursor = Cursor::Available(Point::new(50.0, 50.0));
        let press_event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        let _ = canvas.update(&mut state, &press_event, bounds, empty_cursor);
        assert_eq!(*selected_edge.lock().unwrap(), None);
    }

    #[test]
    fn test_minimap_transform_and_fit_to_view() {
        use uuid::Uuid;

        let node1 = DiagramNode::custom(
            Uuid::new_v4(),
            "Node1".to_string(),
            100.0,
            100.0,
            160.0,
            80.0,
        );
        let node2 = DiagramNode::custom(
            Uuid::new_v4(),
            "Node2".to_string(),
            500.0,
            300.0,
            160.0,
            80.0,
        );
        let nodes = vec![node1, node2];
        let viewport = CanvasViewport::default();
        let screen_size = Size::new(1000.0, 800.0);
        let minimap_rect = Rectangle::new(Point::new(800.0, 600.0), Size::new(160.0, 100.0));

        let transform = MinimapTransform::compute(&nodes, &viewport, screen_size, minimap_rect);
        assert!(transform.scale > 0.0);

        // Bijektion: world -> minimap -> world
        let world_pt = Point::new(300.0, 200.0);
        let mini_pt = transform.world_to_minimap(world_pt);
        assert!(minimap_rect.contains(mini_pt));
        let recovered_world = transform.minimap_to_world(mini_pt);
        assert!((recovered_world.x - world_pt.x).abs() < 0.01);
        assert!((recovered_world.y - world_pt.y).abs() < 0.01);

        // Viewport rect i minimap
        let vp_rect = transform.viewport_rect(&viewport, screen_size);
        assert!(vp_rect.width > 0.0);
        assert!(vp_rect.height > 0.0);

        // Test compute_fit_to_view
        let fit_vp = compute_fit_to_view(&nodes, screen_size);
        assert!(fit_vp.zoom() >= CanvasViewport::MIN_ZOOM && fit_vp.zoom() <= 1.0);
        let world_center = fit_vp.to_world(Point::new(500.0, 400.0));
        // Center af noder: x: (100 + 660)/2 = 380, y: (100 + 380)/2 = 240
        assert!((world_center.x - 380.0).abs() < 1.0);
        assert!((world_center.y - 240.0).abs() < 1.0);
    }
}
