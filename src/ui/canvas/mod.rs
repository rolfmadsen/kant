//! # Canvas Studio Modul
//!
//! Dekomponeret arkitektur for det visuelle diagramlærred i Kant (Task 085):
//! - `viewport`: Koordinattransformation, zoom, pan og fit-to-view
//! - `node_renderer`: Rendering af begreber, UML-klasser, enumerationer og datatyper
//! - `edge_renderer`: Rendering af relationer, pile, multipliciteter og forbindelseslinjer
//! - `minimap`: Svævende kontrolpanel og oversigtskort
//! - `interaction`: Mus- og tastaturinteraktion, drag-select og marquee

pub mod edge_interaction;
pub mod edge_renderer;
pub mod interaction;
pub mod minimap;
pub mod node_renderer;
pub mod traits;
pub mod viewport;

pub use edge_interaction::*;
pub use edge_renderer::*;
pub use interaction::*;
pub use minimap::*;
pub use node_renderer::*;
pub use traits::*;
pub use viewport::*;

use crate::design_system::tokens::ColorTokens;
use crate::features::concept_model::{BendPoint, NodeId, GRID_SIZE};
use crate::ui::theme::ThemeColors;
use iced::mouse;
use iced::widget::canvas::{Action, Event, Frame, Geometry, Path, Program, Stroke};
use iced::{Point, Rectangle, Renderer, Theme};
use std::time::Instant;

pub const CLICK_SLOP_PX: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickTarget {
    Node(NodeId),
    Bend((NodeId, NodeId), usize),
    Empty,
}

#[derive(Debug, Clone, Copy)]
pub struct ClickRecord {
    pub position: Point,
    pub world_position: Point,
    pub time: Instant,
    pub target: ClickTarget,
}

/// Tilstand for rektangulær drag-select (marquee)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarqueeState {
    pub start: Point,
    pub current: Point,
}

impl MarqueeState {
    pub fn select_nodes<N: CanvasNode>(&self, nodes: &[N]) -> std::collections::HashSet<NodeId> {
        let min_x = self.start.x.min(self.current.x);
        let max_x = self.start.x.max(self.current.x);
        let min_y = self.start.y.min(self.current.y);
        let max_y = self.start.y.max(self.current.y);
        nodes
            .iter()
            .filter(|n| {
                let (nx, ny) = n.position();
                let (nw, nh) = n.size();
                nx >= min_x && nx + nw <= max_x && ny >= min_y && ny + nh <= max_y
            })
            .map(|n| n.id())
            .collect()
    }
}

/// Tilstand for synkron parallelforskydning af noder (bulk drag)
#[derive(Debug, Clone, PartialEq)]
pub struct BulkDragState {
    pub leader_id: NodeId,
    pub start_world_pos: Point,
    pub initial_positions: Vec<(NodeId, (f32, f32))>,
    pub last_delta: Option<(f32, f32)>,
    pub start_time: Instant,
}

#[derive(Debug, Clone)]
pub struct DragBendState {
    pub edge: (NodeId, NodeId),
    pub bend_index: usize,
    pub current_bends: Vec<BendPoint>,
}

#[derive(Debug, Clone)]
pub struct DragSegmentState {
    pub edge: (NodeId, NodeId),
    pub segment_index: usize,
    pub is_horizontal: bool,
    pub bend_indices: Vec<usize>,
    pub current_bends: Vec<BendPoint>,
}

#[derive(Debug, Default)]
pub struct DiagramCanvasState {
    pub dragging_node: Option<(NodeId, iced::Vector)>,
    pub bulk_drag: Option<BulkDragState>,
    pub marquee: Option<MarqueeState>,
    pub last_click: Option<ClickRecord>,
    pub pending_double: Option<(ClickTarget, Point)>,
    pub press_screen_pos: Option<Point>,
    pub panning_start: Option<(Point, iced::Vector)>,
    pub is_panning_space: bool,
    pub is_panning_minimap: bool,
    pub modifiers: iced::keyboard::Modifiers,
    pub connecting_from: Option<NodeId>,
    pub connecting_cursor: Option<Point>,
    pub hovered_target_node: Option<NodeId>,
    pub selected_node_ids: std::collections::HashSet<NodeId>,
    pub last_canvas_selection: Option<std::collections::HashSet<NodeId>>,
    pub selected_bend: Option<((NodeId, NodeId), usize)>,
    pub dragging_bend: Option<DragBendState>,
    pub dragging_segment: Option<DragSegmentState>,
}

pub(crate) type EdgeSelectHandler<'a, Message> =
    Box<dyn Fn(Option<(NodeId, NodeId)>) -> Message + 'a>;
pub(crate) type EdgeCreateHandler<'a, Message> = Box<dyn Fn(NodeId, NodeId) -> Message + 'a>;
pub(crate) type EdgeBendsChangeHandler<'a, Message> =
    Box<dyn Fn((NodeId, NodeId), Option<Vec<BendPoint>>) -> Message + 'a>;
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
    pub(crate) nodes: &'a [N],
    pub(crate) edges: &'a [E],
    pub(crate) selected_node_id: Option<NodeId>,
    pub(crate) selected_node_ids: std::collections::HashSet<NodeId>,
    pub(crate) selected_edge: Option<(NodeId, NodeId)>,
    pub(crate) viewport: CanvasViewport,
    pub(crate) is_space_pressed: bool,
    pub(crate) is_palette_dragging: bool,
    pub(crate) render_node: R,
    pub(crate) on_node_selected: Box<dyn Fn(Option<NodeId>) -> Message + 'a>,
    pub(crate) on_selection_changed: Option<SelectionChangeHandler<'a, Message>>,
    pub(crate) on_edge_selected: Option<EdgeSelectHandler<'a, Message>>,
    pub(crate) on_edge_created: Option<EdgeCreateHandler<'a, Message>>,
    pub(crate) on_edge_bends_changed: Option<EdgeBendsChangeHandler<'a, Message>>,
    pub(crate) on_node_moved: Box<dyn Fn(NodeId, f32, f32) -> Message + 'a>,
    pub(crate) on_nodes_moved: Option<NodesMovedHandler<'a, Message>>,
    pub(crate) on_canvas_double_clicked: Box<dyn Fn(f32, f32) -> Message + 'a>,
    pub(crate) on_node_double_clicked: Box<dyn Fn(NodeId) -> Message + 'a>,
    pub(crate) on_viewport_changed: Box<dyn Fn(CanvasViewport) -> Message + 'a>,
    pub(crate) on_canvas_drop: Option<Box<dyn Fn(f32, f32) -> Message + 'a>>,
    pub(crate) on_drag_released: Option<Box<dyn Fn() -> Message + 'a>>,
}

impl<'a, Message, N, E, R> DiagramCanvas<'a, Message, N, E, R>
where
    N: CanvasNode,
    E: CanvasEdge,
    R: Fn(&mut Frame, &N, bool, CanvasViewport) + 'a,
{
    pub const PANEL_WIDTH: f32 = minimap::PANEL_WIDTH;
    pub const PANEL_HEIGHT: f32 = minimap::PANEL_HEIGHT;
    pub const PANEL_MARGIN: f32 = minimap::PANEL_MARGIN;
    pub const MINIMAP_HEIGHT: f32 = minimap::MINIMAP_HEIGHT;

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
            on_edge_bends_changed: None,
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

    pub fn on_edge_bends_changed(
        mut self,
        handler: impl Fn((NodeId, NodeId), Option<Vec<BendPoint>>) -> Message + 'a,
    ) -> Self {
        self.on_edge_bends_changed = Some(Box::new(handler));
        self
    }

    pub fn floating_panel_rect(bounds: Rectangle) -> Rectangle {
        minimap::floating_panel_rect(bounds)
    }

    pub fn minimap_rect(panel_rect: Rectangle) -> Rectangle {
        minimap::minimap_rect(panel_rect)
    }

    pub fn zoom_out_button_rect(panel_rect: Rectangle) -> Rectangle {
        minimap::zoom_out_button_rect(panel_rect)
    }

    pub fn zoom_label_rect(panel_rect: Rectangle) -> Rectangle {
        minimap::zoom_label_rect(panel_rect)
    }

    pub fn zoom_in_button_rect(panel_rect: Rectangle) -> Rectangle {
        minimap::zoom_in_button_rect(panel_rect)
    }

    pub fn fit_view_button_rect(panel_rect: Rectangle) -> Rectangle {
        minimap::fit_view_button_rect(panel_rect)
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
        interaction::handle_canvas_event(self, state, event, bounds, cursor)
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

        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            ColorTokens::CANVAS_BACKGROUND.into_color(),
        );

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
                grid_builder.rectangle(Point::new(gx - 1.0, gy - 1.0), iced::Size::new(2.0, 2.0));
                gy += step;
            }
            gx += step;
        }
        let grid_path = grid_builder.build();
        frame.fill(&grid_path, dot_color);

        frame.translate(self.viewport.pan());
        frame.scale(self.viewport.zoom());

        if self.is_palette_dragging {
            if let Some(cursor_pos) = cursor.position_in(bounds) {
                let world_pos = self.viewport.to_world(cursor_pos);
                let snapped_x = (world_pos.x / GRID_SIZE).round() * GRID_SIZE;
                let snapped_y = (world_pos.y / GRID_SIZE).round() * GRID_SIZE;
                let preview_rect = Path::rounded_rectangle(
                    Point::new(snapped_x, snapped_y),
                    iced::Size::new(140.0, 60.0),
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

        let in_progress_bends = state
            .dragging_bend
            .as_ref()
            .map(|d| (d.edge, d.current_bends.as_slice()))
            .or_else(|| {
                state
                    .dragging_segment
                    .as_ref()
                    .map(|d| (d.edge, d.current_bends.as_slice()))
            });

        draw_edges(
            &mut frame,
            self.nodes,
            self.edges,
            self.selected_edge,
            in_progress_bends,
            state.selected_bend,
        );

        for node in self.nodes {
            let is_selected = self.is_node_selected(node.id());
            (self.render_node)(&mut frame, node, is_selected, self.viewport);

            if state.hovered_target_node == Some(node.id()) {
                draw_node_target_glow(&mut frame, node);
            }

            if is_selected
                && (self.selected_node_id == Some(node.id()) || self.selected_node_ids.len() <= 1)
            {
                draw_node_connect_handle(&mut frame, node);
            }
        }

        draw_connecting_preview(
            &mut frame,
            self.nodes,
            state.connecting_from,
            state.connecting_cursor,
        );

        if let Some(marquee) = state.marquee {
            draw_marquee(&mut frame, marquee);
        }

        let minimap_geom =
            draw_minimap_and_panel(renderer, bounds, self.nodes, &self.viewport, |id| {
                self.is_node_selected(id)
            });

        vec![frame.into_geometry(), minimap_geom]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::concept_model::{DiagramEdge, DiagramNode, RelationKind};
    use iced::mouse::{Button, Cursor};
    use iced::widget::canvas::Event;
    use iced::{Point, Rectangle, Size};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use uuid::Uuid;

    #[test]
    fn test_node_selection_and_dragging_lifecycle() {
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
            false,
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

        let press_event = Event::Mouse(mouse::Event::ButtonPressed(Button::Left));
        let action = canvas.update(&mut state, &press_event, bounds, cursor_on_node);
        assert!(action.is_some());
        assert_eq!(*selected.lock().unwrap(), Some(nodes[0].id()));
        assert!(state.dragging_node.is_some());

        let move_cursor = Cursor::Available(Point::new(160.0, 140.0));
        let move_event = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(160.0, 140.0),
        });
        let action = canvas.update(&mut state, &move_event, bounds, move_cursor);
        assert!(action.is_some());
        assert!(moved.lock().unwrap().is_some());

        let release_event = Event::Mouse(mouse::Event::ButtonReleased(Button::Left));
        let _ = canvas.update(&mut state, &release_event, bounds, move_cursor);
        assert!(state.dragging_node.is_none());
        assert!(state.panning_start.is_none());
    }

    #[test]
    fn test_task_067_diagram_canvas_motion_filtering_and_drag_release_callback() {
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

        let press = Event::Mouse(mouse::Event::ButtonPressed(Button::Left));
        let _ = canvas.update(&mut state, &press, bounds, cursor_start);
        assert!(state.bulk_drag.is_some());
        assert_eq!(moves_count.load(Ordering::SeqCst), 0);

        let cursor_step1 = Cursor::Available(Point::new(140.0, 120.0));
        let move1 = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(140.0, 120.0),
        });
        let _ = canvas.update(&mut state, &move1, bounds, cursor_step1);
        assert_eq!(moves_count.load(Ordering::SeqCst), 1);

        let cursor_subpixel = Cursor::Available(Point::new(142.0, 121.0));
        let move_subpixel = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(142.0, 121.0),
        });
        let _ = canvas.update(&mut state, &move_subpixel, bounds, cursor_subpixel);
        assert_eq!(moves_count.load(Ordering::SeqCst), 1);

        let cursor_step2 = Cursor::Available(Point::new(160.0, 120.0));
        let move2 = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(160.0, 120.0),
        });
        let _ = canvas.update(&mut state, &move2, bounds, cursor_step2);
        assert_eq!(moves_count.load(Ordering::SeqCst), 2);

        let release = Event::Mouse(mouse::Event::ButtonReleased(Button::Left));
        let _ = canvas.update(&mut state, &release, bounds, cursor_step2);
        assert_eq!(release_count.load(Ordering::SeqCst), 1);
        assert!(state.bulk_drag.is_none());
    }

    #[test]
    fn test_drag_to_connect_lifecycle_and_edge_selection() {
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

        let handle_cursor = Cursor::Available(Point::new(280.0, 140.0));
        let press_event = Event::Mouse(mouse::Event::ButtonPressed(Button::Left));
        let action = canvas.update(&mut state, &press_event, bounds, handle_cursor);
        assert!(action.is_some());
        assert_eq!(state.connecting_from, Some(node1.id()));
        assert_eq!(state.connecting_cursor, Some(Point::new(280.0, 140.0)));

        let target_cursor = Cursor::Available(Point::new(450.0, 120.0));
        let move_event = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(450.0, 120.0),
        });
        let action = canvas.update(&mut state, &move_event, bounds, target_cursor);
        assert!(action.is_some());
        assert_eq!(state.hovered_target_node, Some(node2.id()));

        let release_event = Event::Mouse(mouse::Event::ButtonReleased(Button::Left));
        let action = canvas.update(&mut state, &release_event, bounds, target_cursor);
        assert!(action.is_some());
        assert_eq!(
            *created_edge.lock().unwrap(),
            Some((node1.id(), node2.id()))
        );
        assert_eq!(state.connecting_from, None);
        assert_eq!(state.hovered_target_node, None);

        let edge_cursor = Cursor::Available(Point::new(340.0, 140.0));
        let press_event = Event::Mouse(mouse::Event::ButtonPressed(Button::Left));
        let action = canvas.update(&mut state, &press_event, bounds, edge_cursor);
        assert!(action.is_some());
        assert_eq!(
            *selected_edge.lock().unwrap(),
            Some((node1.id(), node2.id()))
        );

        let empty_cursor = Cursor::Available(Point::new(50.0, 50.0));
        let press_event = Event::Mouse(mouse::Event::ButtonPressed(Button::Left));
        let _ = canvas.update(&mut state, &press_event, bounds, empty_cursor);
        assert_eq!(*selected_edge.lock().unwrap(), None);
    }

    #[test]
    fn test_pan_drag_cursor_tracking_does_not_drift() {
        let mut state = DiagramCanvasState::default();
        let initial_pan = iced::Vector::new(10.0, 20.0);
        let click_pos = Point::new(100.0, 100.0);
        state.panning_start = Some((click_pos, initial_pan));

        let moves = [
            Point::new(120.0, 100.0),
            Point::new(150.0, 100.0),
            Point::new(200.0, 100.0),
        ];

        let mut last_pan = iced::Vector::default();
        for current_cursor in moves {
            let (start_pos, init_pan) = state.panning_start.unwrap();
            let total_delta = current_cursor - start_pos;
            let current_pan = init_pan + total_delta;
            last_pan = current_pan;
        }

        assert_eq!(last_pan, iced::Vector::new(110.0, 20.0));
    }
}
