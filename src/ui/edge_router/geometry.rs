use crate::features::concept_model::{BendPoint, PortSide};
use crate::ui::edge_router::types::{
    ArrowHead, BridgeHop, Diamond, HalfArrow, RoutedEdge, ARROW_HEAD_LENGTH, ARROW_HEAD_WIDTH,
    CHANNEL_OFFSET, DIAMOND_LENGTH, DIAMOND_WIDTH,
};
use iced::Point;

pub fn compute_half_arrow(target_boundary: Point, side: PortSide) -> HalfArrow {
    let len = 10.0;
    let w = 6.0;

    let barb = match side {
        PortSide::Left => Point::new(target_boundary.x - len, target_boundary.y - w),
        PortSide::Right => Point::new(target_boundary.x + len, target_boundary.y - w),
        PortSide::Top => Point::new(target_boundary.x - w, target_boundary.y - len),
        PortSide::Bottom => Point::new(target_boundary.x - w, target_boundary.y + len),
    };

    HalfArrow {
        tip: target_boundary,
        barb,
        direction: side,
    }
}

pub fn compute_diamond(source_boundary: Point, side: PortSide) -> Diamond {
    let half_w = DIAMOND_WIDTH / 2.0;
    let len = DIAMOND_LENGTH;
    let half_len = len / 2.0;

    match side {
        PortSide::Right => Diamond {
            tip: source_boundary,
            left: Point::new(source_boundary.x + half_len, source_boundary.y - half_w),
            right: Point::new(source_boundary.x + half_len, source_boundary.y + half_w),
            back: Point::new(source_boundary.x + len, source_boundary.y),
            direction: PortSide::Right,
        },
        PortSide::Left => Diamond {
            tip: source_boundary,
            left: Point::new(source_boundary.x - half_len, source_boundary.y - half_w),
            right: Point::new(source_boundary.x - half_len, source_boundary.y + half_w),
            back: Point::new(source_boundary.x - len, source_boundary.y),
            direction: PortSide::Left,
        },
        PortSide::Top => Diamond {
            tip: source_boundary,
            left: Point::new(source_boundary.x - half_w, source_boundary.y - half_len),
            right: Point::new(source_boundary.x + half_w, source_boundary.y - half_len),
            back: Point::new(source_boundary.x, source_boundary.y - len),
            direction: PortSide::Top,
        },
        PortSide::Bottom => Diamond {
            tip: source_boundary,
            left: Point::new(source_boundary.x - half_w, source_boundary.y + half_len),
            right: Point::new(source_boundary.x + half_w, source_boundary.y + half_len),
            back: Point::new(source_boundary.x, source_boundary.y + len),
            direction: PortSide::Bottom,
        },
    }
}

pub fn compute_arrow_head(target_boundary: Point, side: PortSide) -> ArrowHead {
    let half_w = ARROW_HEAD_WIDTH / 2.0;
    let len = ARROW_HEAD_LENGTH;

    match side {
        PortSide::Bottom => ArrowHead {
            tip: target_boundary,
            left: Point::new(target_boundary.x - half_w, target_boundary.y + len),
            right: Point::new(target_boundary.x + half_w, target_boundary.y + len),
            direction: PortSide::Bottom,
        },
        PortSide::Top => ArrowHead {
            tip: target_boundary,
            left: Point::new(target_boundary.x - half_w, target_boundary.y - len),
            right: Point::new(target_boundary.x + half_w, target_boundary.y - len),
            direction: PortSide::Top,
        },
        PortSide::Left => ArrowHead {
            tip: target_boundary,
            left: Point::new(target_boundary.x - len, target_boundary.y - half_w),
            right: Point::new(target_boundary.x - len, target_boundary.y + half_w),
            direction: PortSide::Left,
        },
        PortSide::Right => ArrowHead {
            tip: target_boundary,
            left: Point::new(target_boundary.x + len, target_boundary.y - half_w),
            right: Point::new(target_boundary.x + len, target_boundary.y + half_w),
            direction: PortSide::Right,
        },
    }
}

pub fn build_orthogonal_path(
    start: Point,
    end: Point,
    start_side: PortSide,
    end_side: PortSide,
    channel_index: usize,
) -> Vec<Point> {
    let channel_offset = (channel_index % 3) as f32 * CHANNEL_OFFSET;
    let mut pts = vec![start];

    let stub_len = 16.0 + channel_offset;

    // Vinkelret stub ud fra start
    let (snx, sny) = start_side.normal();
    let start_stub = Point::new(start.x + snx * stub_len, start.y + sny * stub_len);

    // Vinkelret stub ud fra end
    let (enx, eny) = end_side.normal();
    let end_stub = Point::new(end.x + enx * stub_len, end.y + eny * stub_len);

    if (start_side == PortSide::Top && end_side == PortSide::Bottom)
        || (start_side == PortSide::Bottom && end_side == PortSide::Top)
    {
        let mid_y = (start_stub.y + end_stub.y) / 2.0;
        pts.push(Point::new(start.x, mid_y));
        pts.push(Point::new(end.x, mid_y));
    } else if (start_side == PortSide::Right && end_side == PortSide::Left)
        || (start_side == PortSide::Left && end_side == PortSide::Right)
    {
        let mid_x = (start_stub.x + end_stub.x) / 2.0;
        pts.push(Point::new(mid_x, start.y));
        pts.push(Point::new(mid_x, end.y));
    } else if start_side == PortSide::Right && end_side == PortSide::Right {
        // C-bøjning på højre side
        let max_x = start_stub.x.max(end_stub.x) + 20.0 + channel_offset;
        pts.push(Point::new(max_x, start.y));
        pts.push(Point::new(max_x, end.y));
    } else if start_side == PortSide::Left && end_side == PortSide::Left {
        // C-bøjning på venstre side
        let min_x = start_stub.x.min(end_stub.x) - 20.0 - channel_offset;
        pts.push(Point::new(min_x, start.y));
        pts.push(Point::new(min_x, end.y));
    } else if start_side.is_vertical() && !end_side.is_vertical() {
        // Start vertikal, end horisontal
        pts.push(Point::new(start.x, end.y));
    } else if !start_side.is_vertical() && end_side.is_vertical() {
        // Start horisontal, end vertikal
        pts.push(Point::new(end.x, start.y));
    } else {
        // Generelt fallback med 2 hjørner via stubs
        pts.push(start_stub);
        pts.push(Point::new(end_stub.x, start_stub.y));
        pts.push(end_stub);
    }

    pts.push(end);
    simplify_path(pts)
}

pub fn build_orthogonal_path_with_bends(
    start: Point,
    end: Point,
    start_side: PortSide,
    end_side: PortSide,
    bends: &[BendPoint],
) -> Vec<Point> {
    if bends.is_empty() {
        return build_orthogonal_path(start, end, start_side, end_side, 0);
    }

    let mut pts = Vec::new();
    pts.push(start);

    let p1 = Point::new(bends[0].x, bends[0].y);

    // 1. Terminal start segment til P_1
    if start_side.is_vertical() {
        if (start.x - p1.x).abs() < 0.5 {
            pts.push(p1);
        } else {
            pts.push(Point::new(start.x, p1.y));
            pts.push(p1);
        }
    } else {
        if (start.y - p1.y).abs() < 0.5 {
            pts.push(p1);
        } else {
            pts.push(Point::new(p1.x, start.y));
            pts.push(p1);
        }
    }

    // 2. Mellemliggende knækpunkter
    for i in 0..bends.len() - 1 {
        let curr = Point::new(bends[i].x, bends[i].y);
        let next = Point::new(bends[i + 1].x, bends[i + 1].y);

        if (curr.x - next.x).abs() < 0.5 || (curr.y - next.y).abs() < 0.5 {
            pts.push(next);
        } else {
            let last_pt = *pts.last().unwrap();
            let was_vertical = (last_pt.x - curr.x).abs() < 0.5;
            if was_vertical {
                pts.push(Point::new(next.x, curr.y));
            } else {
                pts.push(Point::new(curr.x, next.y));
            }
            pts.push(next);
        }
    }

    // 3. Terminal slut segment fra P_k til end
    let pk = Point::new(bends[bends.len() - 1].x, bends[bends.len() - 1].y);
    if end_side.is_vertical() {
        if (pk.x - end.x).abs() < 0.5 {
            pts.push(end);
        } else {
            pts.push(Point::new(end.x, pk.y));
            pts.push(end);
        }
    } else {
        if (pk.y - end.y).abs() < 0.5 {
            pts.push(end);
        } else {
            pts.push(Point::new(pk.x, end.y));
            pts.push(end);
        }
    }

    simplify_path_preserving_bends(pts, bends)
}

pub fn simplify_path_preserving_bends(points: Vec<Point>, bends: &[BendPoint]) -> Vec<Point> {
    if points.len() <= 2 {
        return points;
    }

    let is_bend = |p: Point| -> bool {
        bends
            .iter()
            .any(|b| (b.x - p.x).abs() < 0.1 && (b.y - p.y).abs() < 0.1)
    };

    let mut simplified = Vec::new();
    simplified.push(points[0]);

    for i in 1..points.len() - 1 {
        let prev = *simplified.last().unwrap();
        let curr = points[i];
        let next = points[i + 1];

        if is_bend(curr) {
            simplified.push(curr);
            continue;
        }

        let collinear_h = (prev.y - curr.y).abs() < 0.001 && (curr.y - next.y).abs() < 0.001;
        let collinear_v = (prev.x - curr.x).abs() < 0.001 && (curr.x - next.x).abs() < 0.001;

        if !collinear_h && !collinear_v {
            simplified.push(curr);
        }
    }

    simplified.push(*points.last().unwrap());
    simplified
}

pub fn simplify_path(points: Vec<Point>) -> Vec<Point> {
    if points.len() <= 2 {
        return points;
    }

    let mut simplified = Vec::new();
    simplified.push(points[0]);

    for i in 1..points.len() - 1 {
        let prev = *simplified.last().unwrap();
        let curr = points[i];
        let next = points[i + 1];

        let collinear_h = (prev.y - curr.y).abs() < 0.001 && (curr.y - next.y).abs() < 0.001;
        let collinear_v = (prev.x - curr.x).abs() < 0.001 && (curr.x - next.x).abs() < 0.001;

        if !collinear_h && !collinear_v {
            simplified.push(curr);
        }
    }

    simplified.push(*points.last().unwrap());
    simplified
}

/// Sletter overflødige kollineære knækpunkter og mikrosegmenter ved drop-commit
pub fn prune_collinear_bends(bends: &[BendPoint]) -> Vec<BendPoint> {
    if bends.is_empty() {
        return Vec::new();
    }
    if bends.len() == 1 {
        return bends.to_vec();
    }
    if bends.len() == 2 {
        if (bends[0].x - bends[1].x).hypot(bends[0].y - bends[1].y) < 4.0 {
            return Vec::new();
        }
        return bends.to_vec();
    }

    let mut result = Vec::new();
    result.push(bends[0]);

    for i in 1..bends.len() - 1 {
        let prev = *result.last().unwrap();
        let curr = bends[i];
        let next = bends[i + 1];

        let collinear_h = (prev.y - curr.y).abs() < 0.5 && (curr.y - next.y).abs() < 0.5;
        let collinear_v = (prev.x - curr.x).abs() < 0.5 && (curr.x - next.x).abs() < 0.5;
        let is_micro = (prev.x - curr.x).hypot(prev.y - curr.y) < 4.0;

        if !collinear_h && !collinear_v && !is_micro {
            result.push(curr);
        }
    }

    let last = *bends.last().unwrap();
    if let Some(prev) = result.last() {
        if (prev.x - last.x).hypot(prev.y - last.y) >= 4.0 {
            result.push(last);
        }
    } else {
        result.push(last);
    }

    if result.len() == 2 && (result[0].x - result[1].x).hypot(result[0].y - result[1].y) < 4.0 {
        return Vec::new();
    }

    result
}

pub fn compute_label_pos(points: &[Point]) -> Option<Point> {
    if points.len() < 2 {
        return None;
    }

    let mut best_len = 0.0;
    let mut best_pos = None;

    for window in points.windows(2) {
        let p1 = window[0];
        let p2 = window[1];
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let len = (dx * dx + dy * dy).sqrt();

        if len > best_len {
            best_len = len;
            let mid_x = (p1.x + p2.x) / 2.0;
            let mid_y = (p1.y + p2.y) / 2.0;
            let offset_y = if (p1.y - p2.y).abs() < 0.001 {
                -10.0
            } else {
                0.0
            };
            best_pos = Some(Point::new(mid_x, mid_y + offset_y));
        }
    }

    best_pos
}

pub fn detect_bridges(routes: &mut [RoutedEdge]) {
    let n = routes.len();
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }

            let mut bridges = Vec::new();
            for w_h in routes[i].points.windows(2) {
                let (p1, p2) = (w_h[0], w_h[1]);
                if (p1.y - p2.y).abs() > 0.001 {
                    continue;
                }

                let h_y = p1.y;
                let h_min_x = p1.x.min(p2.x);
                let h_max_x = p1.x.max(p2.x);

                for w_v in routes[j].points.windows(2) {
                    let (q1, q2) = (w_v[0], w_v[1]);
                    if (q1.x - q2.x).abs() > 0.001 {
                        continue;
                    }

                    let v_x = q1.x;
                    let v_min_y = q1.y.min(q2.y);
                    let v_max_y = q1.y.max(q2.y);

                    if v_x > h_min_x + 2.0
                        && v_x < h_max_x - 2.0
                        && h_y > v_min_y + 2.0
                        && h_y < v_max_y - 2.0
                    {
                        bridges.push(BridgeHop {
                            center: Point::new(v_x, h_y),
                            is_horizontal: true,
                        });
                    }
                }
            }

            routes[i].bridges.extend(bridges);
        }
    }
}
