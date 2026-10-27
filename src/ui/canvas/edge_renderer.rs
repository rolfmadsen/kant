use super::{CanvasEdge, CanvasNode};
use crate::design_system::tokens::ColorTokens;
use crate::features::concept_model::{
    BendPoint, DiagramEdge, DiagramNode, NodeId, PortSide, RelationKind,
};
use crate::ui::edge_router::EdgeRouter;
use crate::ui::theme::ThemeColors;
use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{alignment, Color, Point, Size};

/// Afstand fra punkt p til linjesegment a-b
pub fn distance_to_segment(p: Point, a: Point, b: Point) -> f32 {
    let l2 = (b.x - a.x).powi(2) + (b.y - a.y).powi(2);
    if l2 < 0.0001 {
        return (p.x - a.x).hypot(p.y - a.y);
    }
    let t = (((p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y)) / l2).clamp(0.0, 1.0);
    let projection = Point::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y));
    (p.x - projection.x).hypot(p.y - projection.y)
}

/// Beregner placering af multiplicitetstekst ved port
pub fn port_multiplicity_pos(pt: Point, side: PortSide) -> Point {
    match side {
        PortSide::Right => Point::new(pt.x + 14.0, pt.y - 10.0),
        PortSide::Left => Point::new(pt.x - 14.0, pt.y - 10.0),
        PortSide::Top => Point::new(pt.x + 10.0, pt.y - 12.0),
        PortSide::Bottom => Point::new(pt.x + 10.0, pt.y + 12.0),
    }
}

/// Renderer multiplicitetstekst med baggrundspille
pub fn render_multiplicity_label(frame: &mut Frame, text: &str, pos: Point, is_selected: bool) {
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

/// Tegner ortogonale kanter, linjesegmenter med broer, pilehoveder og labels
pub fn draw_edges<N: CanvasNode, E: CanvasEdge>(
    frame: &mut Frame,
    nodes: &[N],
    edges: &[E],
    selected_edge: Option<(NodeId, NodeId)>,
    in_progress_bends: Option<((NodeId, NodeId), &[BendPoint])>,
    selected_bend: Option<((NodeId, NodeId), usize)>,
) {
    let diagram_nodes: Vec<DiagramNode> = nodes.iter().map(|n| n.to_diagram_node()).collect();
    let mut diagram_edges: Vec<DiagramEdge> = edges.iter().map(|e| e.to_diagram_edge()).collect();

    if let Some(((b_from, b_to), bends)) = in_progress_bends {
        for de in &mut diagram_edges {
            if (de.from() == b_from && de.to() == b_to) || (de.from() == b_to && de.to() == b_from)
            {
                de.set_bends(Some(bends.to_vec()));
            }
        }
    }

    let routed_edges = EdgeRouter::route_edges(&diagram_nodes, &diagram_edges);

    // 1. Linjesegmenter med line jumps (broer)
    for routed in &routed_edges {
        if routed.points.len() < 2 {
            continue;
        }

        let is_selected_edge = selected_edge == Some((routed.from, routed.to))
            || selected_edge == Some((routed.to, routed.from));
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
                        seg_bridges.sort_by(|a, b| a.center.x.partial_cmp(&b.center.x).unwrap());
                    } else {
                        seg_bridges.sort_by(|a, b| b.center.x.partial_cmp(&a.center.x).unwrap());
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

    // 2. Pilehoveder og labels
    for routed in &routed_edges {
        let is_selected_edge = selected_edge == Some((routed.from, routed.to))
            || selected_edge == Some((routed.to, routed.from));
        let arrow_stroke_color = if is_selected_edge {
            ThemeColors::PRIMARY
        } else {
            ColorTokens::SLATE_800.into_color()
        };
        let arrow_stroke_width = if is_selected_edge { 2.2 } else { 1.5 };

        // Kompositions-diamant ved kilde-noden
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

        // Pilehoved ved mål-noden
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

        // Halv pil for rettet association
        if let Some(ref half_arrow) = routed.half_arrow {
            let barb_line = Path::line(half_arrow.tip, half_arrow.barb);
            frame.stroke(
                &barb_line,
                Stroke::default()
                    .with_color(arrow_stroke_color)
                    .with_width(arrow_stroke_width + 0.5),
            );
        }

        // Kantlabel
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

        // Multipliciteter ved kilde- og målporte
        if let Some(edge) = edges
            .iter()
            .find(|e| e.from() == routed.from && e.to() == routed.to && e.kind() == routed.kind)
        {
            if let Some(src_mult) = edge.source_multiplicity() {
                let p_src = routed.points[0];
                let pos = port_multiplicity_pos(p_src, routed.from_side);
                render_multiplicity_label(frame, &src_mult, pos, is_selected_edge);
            }

            if let Some(tgt_mult) = edge.target_multiplicity() {
                let p_tgt = *routed.points.last().unwrap_or(&routed.points[0]);
                let pos = port_multiplicity_pos(p_tgt, routed.to_side);
                render_multiplicity_label(frame, &tgt_mult, pos, is_selected_edge);
            }
        }
    }

    // 3. Knækpunkthåndtag for valgt relation
    for routed in &routed_edges {
        let is_selected_edge = selected_edge == Some((routed.from, routed.to))
            || selected_edge == Some((routed.to, routed.from));
        if is_selected_edge {
            let current_bends: Option<Vec<BendPoint>> =
                if let Some(((b_from, b_to), bends)) = in_progress_bends {
                    if (routed.from == b_from && routed.to == b_to)
                        || (routed.from == b_to && routed.to == b_from)
                    {
                        Some(bends.to_vec())
                    } else {
                        edges
                            .iter()
                            .find(|e| {
                                (e.from() == routed.from && e.to() == routed.to)
                                    || (e.from() == routed.to && e.to() == routed.from)
                            })
                            .and_then(|e| e.bends())
                            .map(|b| b.to_vec())
                    }
                } else {
                    edges
                        .iter()
                        .find(|e| {
                            (e.from() == routed.from && e.to() == routed.to)
                                || (e.from() == routed.to && e.to() == routed.from)
                        })
                        .and_then(|e| e.bends())
                        .map(|b| b.to_vec())
                };

            if let Some(bends) = current_bends {
                for (idx, bend) in bends.iter().enumerate() {
                    let pt = Point::new(bend.x, bend.y);
                    let is_selected = selected_bend == Some(((routed.from, routed.to), idx))
                        || selected_bend == Some(((routed.to, routed.from), idx));
                    let circle = Path::circle(pt, 5.0);
                    frame.fill(
                        &circle,
                        if is_selected {
                            ThemeColors::PRIMARY
                        } else {
                            Color::WHITE
                        },
                    );
                    frame.stroke(
                        &circle,
                        Stroke::default()
                            .with_color(ThemeColors::PRIMARY)
                            .with_width(2.0),
                    );
                }
            }
        }
    }
}

/// Tegner elastik / preview-linje under drag-to-connect
pub fn draw_connecting_preview<N: CanvasNode>(
    frame: &mut Frame,
    nodes: &[N],
    connecting_from: Option<NodeId>,
    connecting_cursor: Option<Point>,
) {
    if let (Some(source_id), Some(cursor_world)) = (connecting_from, connecting_cursor) {
        if let Some(src) = nodes.iter().find(|n| n.id() == source_id) {
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
}

/// Forbindelseshåndtag på primær valgt node
pub fn draw_node_connect_handle(frame: &mut Frame, node: &impl CanvasNode) {
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

/// Highlight målnode under drag-to-connect
pub fn draw_node_target_glow(frame: &mut Frame, node: &impl CanvasNode) {
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

/// Rektangulær drag-select markeringsramme (Marquee)
pub fn draw_marquee(frame: &mut Frame, marquee: super::interaction::MarqueeState) {
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
