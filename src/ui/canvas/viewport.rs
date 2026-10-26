use super::CanvasNode;
use iced::widget::canvas::Action;
use iced::{Point, Size, Vector};

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

    /// Centrerer et målrektangel i verden (target_x, target_y, target_w, target_h)
    /// på skærmens synlige centrum (screen_center) med bevarelse af det aktuelle zoom-niveau.
    pub fn center_on_rect(
        &mut self,
        target_x: f32,
        target_y: f32,
        target_w: f32,
        target_h: f32,
        screen_center: Point,
    ) {
        let world_center_x = target_x + target_w / 2.0;
        let world_center_y = target_y + target_h / 2.0;
        self.pan = Vector::new(
            screen_center.x - world_center_x * self.zoom,
            screen_center.y - world_center_y * self.zoom,
        );
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

/// Håndterer zoom og panorering ved musehjuls-scrolling
pub fn handle_wheel_scroll<Message>(
    viewport: &mut CanvasViewport,
    modifiers: iced::keyboard::Modifiers,
    delta: iced::mouse::ScrollDelta,
    cursor_pos: Point,
    on_viewport_changed: &impl Fn(CanvasViewport) -> Message,
) -> Option<Action<Message>> {
    let (dx, dy) = match delta {
        iced::mouse::ScrollDelta::Lines { x, y } => (x * 30.0, y * 30.0),
        iced::mouse::ScrollDelta::Pixels { x, y } => (x, y),
    };

    if modifiers.control() || modifiers.command() {
        let zoom_factor: f32 = if dy > 0.0 {
            1.1
        } else if dy < 0.0 {
            1.0 / 1.1
        } else {
            1.0
        };
        if (zoom_factor - 1.0f32).abs() > 0.001 {
            viewport.zoom_at(cursor_pos, zoom_factor);
            return Some(Action::publish(on_viewport_changed(*viewport)).and_capture());
        }
    } else if modifiers.shift() {
        let shift_dx = if dy.abs() > dx.abs() { dy } else { dx };
        viewport.translate(Vector::new(shift_dx, 0.0));
        return Some(Action::publish(on_viewport_changed(*viewport)).and_capture());
    } else {
        viewport.translate(Vector::new(dx, dy));
        return Some(Action::publish(on_viewport_changed(*viewport)).and_capture());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::concept_model::DiagramNode;
    use uuid::Uuid;

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
    fn test_compute_fit_to_view() {
        let node1 = DiagramNode::custom(Uuid::new_v4(), "N1".into(), 100.0, 100.0, 160.0, 80.0);
        let node2 = DiagramNode::custom(Uuid::new_v4(), "N2".into(), 500.0, 300.0, 160.0, 80.0);
        let nodes = vec![node1, node2];
        let screen_size = Size::new(1000.0, 800.0);

        let fit_vp = compute_fit_to_view(&nodes, screen_size);
        assert!(fit_vp.zoom() >= CanvasViewport::MIN_ZOOM && fit_vp.zoom() <= 1.0);
        let world_center = fit_vp.to_world(Point::new(500.0, 400.0));
        assert!((world_center.x - 380.0).abs() < 1.0);
        assert!((world_center.y - 240.0).abs() < 1.0);
    }
}
