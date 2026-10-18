use super::viewport::{compute_fit_to_view, CanvasViewport};
use super::CanvasNode;
use crate::design_system::tokens::ColorTokens;
use crate::ui::theme::ThemeColors;
use iced::widget::canvas::{Action, Frame, Geometry, Path, Stroke, Text};
use iced::{alignment, Point, Rectangle, Renderer, Size, Vector};

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
        let (mut min_x, mut max_x) = (vp_tl.x.min(vp_br.x), vp_tl.x.max(vp_br.x));
        let (mut min_y, mut max_y) = (vp_tl.y.min(vp_br.y), vp_tl.y.max(vp_br.y));

        for node in nodes {
            let ((nx, ny), (nw, nh)) = (node.position(), node.size());
            min_x = min_x.min(nx);
            min_y = min_y.min(ny);
            max_x = max_x.max(nx + nw);
            max_y = max_y.max(ny + nh);
        }

        let margin = 60.0;
        let (min_x, min_y) = (min_x - margin, min_y - margin);
        let (max_x, max_y) = (max_x + margin, max_y + margin);
        let world_w = (max_x - min_x).max(100.0);
        let world_h = (max_y - min_y).max(100.0);

        let scale = (minimap_rect.width / world_w).min(minimap_rect.height / world_h);
        let offset_x = minimap_rect.x + (minimap_rect.width - world_w * scale) / 2.0;
        let offset_y = minimap_rect.y + (minimap_rect.height - world_h * scale) / 2.0;

        Self {
            world_bounds: Rectangle::new(Point::new(min_x, min_y), Size::new(world_w, world_h)),
            minimap_rect,
            scale,
            offset: Point::new(offset_x, offset_y),
        }
    }

    pub fn world_to_minimap(&self, p: Point) -> Point {
        Point::new(
            self.offset.x + (p.x - self.world_bounds.x) * self.scale,
            self.offset.y + (p.y - self.world_bounds.y) * self.scale,
        )
    }

    pub fn minimap_to_world(&self, p: Point) -> Point {
        Point::new(
            self.world_bounds.x + (p.x - self.offset.x) / self.scale,
            self.world_bounds.y + (p.y - self.offset.y) / self.scale,
        )
    }

    pub fn viewport_rect(&self, viewport: &CanvasViewport, screen_bounds: Size) -> Rectangle {
        let m_tl = self.world_to_minimap(viewport.to_world(Point::ORIGIN));
        let m_w = (screen_bounds.width / viewport.zoom()) * self.scale;
        let m_h = (screen_bounds.height / viewport.zoom()) * self.scale;
        Rectangle::new(m_tl, Size::new(m_w, m_h))
    }
}

pub const PANEL_WIDTH: f32 = 180.0;
pub const PANEL_HEIGHT: f32 = 148.0;
pub const PANEL_MARGIN: f32 = 16.0;
pub const MINIMAP_HEIGHT: f32 = 104.0;

pub fn floating_panel_rect(b: Rectangle) -> Rectangle {
    Rectangle::new(
        Point::new(
            b.width - PANEL_WIDTH - PANEL_MARGIN,
            b.height - PANEL_HEIGHT - PANEL_MARGIN,
        ),
        Size::new(PANEL_WIDTH, PANEL_HEIGHT),
    )
}

pub fn minimap_rect(p: Rectangle) -> Rectangle {
    Rectangle::new(
        Point::new(p.x + 6.0, p.y + 6.0),
        Size::new(p.width - 12.0, MINIMAP_HEIGHT),
    )
}

pub fn zoom_out_button_rect(p: Rectangle) -> Rectangle {
    Rectangle::new(Point::new(p.x + 8.0, p.y + 116.0), Size::new(26.0, 24.0))
}

pub fn zoom_label_rect(p: Rectangle) -> Rectangle {
    Rectangle::new(Point::new(p.x + 38.0, p.y + 116.0), Size::new(48.0, 24.0))
}

pub fn zoom_in_button_rect(p: Rectangle) -> Rectangle {
    Rectangle::new(Point::new(p.x + 90.0, p.y + 116.0), Size::new(26.0, 24.0))
}

pub fn fit_view_button_rect(p: Rectangle) -> Rectangle {
    Rectangle::new(Point::new(p.x + 120.0, p.y + 116.0), Size::new(52.0, 24.0))
}

fn pan_to_minimap_point<N: CanvasNode>(
    bounds: Rectangle,
    point: Point,
    nodes: &[N],
    viewport: &mut CanvasViewport,
    m_rect: Rectangle,
) {
    let world_pt =
        MinimapTransform::compute(nodes, viewport, bounds.size(), m_rect).minimap_to_world(point);
    viewport.set_pan(Vector::new(
        bounds.width / 2.0 - world_pt.x * viewport.zoom(),
        bounds.height / 2.0 - world_pt.y * viewport.zoom(),
    ));
}

/// Håndterer klik på det svævende ergonomi-kontrolpanel og miniaturekort
pub fn handle_panel_click<N: CanvasNode, Message>(
    bounds: Rectangle,
    cursor_pos: Point,
    nodes: &[N],
    viewport: &mut CanvasViewport,
    is_panning_minimap: &mut bool,
    on_viewport_changed: &impl Fn(CanvasViewport) -> Message,
) -> Option<Action<Message>> {
    let panel_rect = floating_panel_rect(bounds);
    if !panel_rect.contains(cursor_pos) {
        return None;
    }

    if zoom_in_button_rect(panel_rect).contains(cursor_pos) {
        viewport.zoom_at(Point::new(bounds.width / 2.0, bounds.height / 2.0), 1.15);
        return Some(Action::publish(on_viewport_changed(*viewport)).and_capture());
    }
    if zoom_out_button_rect(panel_rect).contains(cursor_pos) {
        viewport.zoom_at(
            Point::new(bounds.width / 2.0, bounds.height / 2.0),
            1.0 / 1.15,
        );
        return Some(Action::publish(on_viewport_changed(*viewport)).and_capture());
    }
    if fit_view_button_rect(panel_rect).contains(cursor_pos)
        || zoom_label_rect(panel_rect).contains(cursor_pos)
    {
        *viewport = compute_fit_to_view(nodes, bounds.size());
        return Some(Action::publish(on_viewport_changed(*viewport)).and_capture());
    }
    let m_rect = minimap_rect(panel_rect);
    if m_rect.contains(cursor_pos) {
        *is_panning_minimap = true;
        pan_to_minimap_point(bounds, cursor_pos, nodes, viewport, m_rect);
        return Some(Action::publish(on_viewport_changed(*viewport)).and_capture());
    }
    Some(Action::capture())
}

/// Håndterer panorering ved træk i miniaturekort
pub fn handle_minimap_drag<N: CanvasNode, Message>(
    bounds: Rectangle,
    cursor_pos: Point,
    nodes: &[N],
    viewport: &mut CanvasViewport,
    on_viewport_changed: &impl Fn(CanvasViewport) -> Message,
) -> Action<Message> {
    let m_rect = minimap_rect(floating_panel_rect(bounds));
    let clamped_pos = Point::new(
        cursor_pos.x.clamp(m_rect.x, m_rect.x + m_rect.width),
        cursor_pos.y.clamp(m_rect.y, m_rect.y + m_rect.height),
    );
    pan_to_minimap_point(bounds, clamped_pos, nodes, viewport, m_rect);
    Action::publish(on_viewport_changed(*viewport)).and_capture()
}

/// Renderer minimap og svævende ergonomi-kontrolpanel
pub fn draw_minimap_and_panel<
    N: CanvasNode,
    F: Fn(crate::features::concept_model::NodeId) -> bool,
>(
    renderer: &Renderer,
    bounds: Rectangle,
    nodes: &[N],
    viewport: &CanvasViewport,
    is_node_selected: F,
) -> Geometry {
    let mut frame = Frame::new(renderer, bounds.size());
    let panel = floating_panel_rect(bounds);
    let mbox = minimap_rect(panel);

    // A. Panelets baggrund og skygge
    let shadow = Path::rounded_rectangle(
        Point::new(panel.x, panel.y + 3.0),
        panel.size(),
        10.0.into(),
    );
    frame.fill(
        &shadow,
        ColorTokens::SLATE_900.with_alpha(0.12).into_color(),
    );
    let ppath = Path::rounded_rectangle(Point::new(panel.x, panel.y), panel.size(), 10.0.into());
    frame.fill(&ppath, ColorTokens::SURFACE.with_alpha(0.92).into_color());
    frame.stroke(
        &ppath,
        Stroke::default()
            .with_color(ThemeColors::SLATE_200)
            .with_width(1.0),
    );

    // B. Minimap baggrund
    let mbg = Path::rounded_rectangle(Point::new(mbox.x, mbox.y), mbox.size(), 6.0.into());
    frame.fill(
        &mbg,
        ColorTokens::CANVAS_BACKGROUND.with_alpha(0.95).into_color(),
    );
    let stroke_slate = Stroke::default()
        .with_color(ColorTokens::SLATE_200.with_alpha(0.80).into_color())
        .with_width(0.8);
    frame.stroke(&mbg, stroke_slate);

    // C. Noder i minimappet
    let transform = MinimapTransform::compute(nodes, viewport, bounds.size(), mbox);
    for node in nodes {
        let ((nx, ny), (nw, nh)) = (node.position(), node.size());
        let tl = transform.world_to_minimap(Point::new(nx, ny));
        let (mini_w, mini_h) = (
            (nw * transform.scale).max(3.0),
            (nh * transform.scale).max(3.0),
        );
        let nrect = Path::rounded_rectangle(tl, Size::new(mini_w, mini_h), 1.5.into());
        let color = if is_node_selected(node.id()) {
            ThemeColors::PRIMARY
        } else {
            ColorTokens::SLATE_400.into_color()
        };
        frame.fill(&nrect, color);
    }

    // D. Viewport ramme i minimappet
    let vp_box = transform.viewport_rect(viewport, bounds.size());
    let vp_path =
        Path::rounded_rectangle(Point::new(vp_box.x, vp_box.y), vp_box.size(), 2.0.into());
    frame.fill(&vp_path, ColorTokens::SELECTION_VIEWPORT.into_color());
    frame.stroke(
        &vp_path,
        Stroke::default()
            .with_color(ThemeColors::PRIMARY)
            .with_width(1.2),
    );

    // E. Knapper i værktøjslinjen (–, 100%, +, ⊡ Fit)
    let draw_btn = |f: &mut Frame, r: Rectangle, label: &str, fsize: f32, is_accent: bool| {
        let bpath = Path::rounded_rectangle(Point::new(r.x, r.y), r.size(), 5.0.into());
        let (bg, sc, tc) = if is_accent {
            (
                ColorTokens::SELECTION_FILL.into_color(),
                ThemeColors::PRIMARY,
                ThemeColors::PRIMARY,
            )
        } else {
            (
                ColorTokens::SLATE_100.with_alpha(0.90).into_color(),
                ThemeColors::SLATE_200,
                ThemeColors::SLATE_800,
            )
        };
        f.fill(&bpath, bg);
        f.stroke(&bpath, Stroke::default().with_color(sc).with_width(0.8));
        f.fill_text(Text {
            content: label.to_string(),
            position: Point::new(r.x + r.width / 2.0, r.y + r.height / 2.0),
            color: tc,
            size: fsize.into(),
            align_x: alignment::Horizontal::Center.into(),
            align_y: alignment::Vertical::Center,
            ..Default::default()
        });
    };

    let zoom_pct = (viewport.zoom() * 100.0).round() as u32;
    let zoom_str = format!("{}%", zoom_pct);
    draw_btn(&mut frame, zoom_out_button_rect(panel), "–", 14.0, false);
    draw_btn(&mut frame, zoom_label_rect(panel), &zoom_str, 11.0, false);
    draw_btn(&mut frame, zoom_in_button_rect(panel), "+", 14.0, false);
    draw_btn(&mut frame, fit_view_button_rect(panel), "⊡ Fit", 11.0, true);

    frame.into_geometry()
}
