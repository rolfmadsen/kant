use super::minimap;
use super::viewport::{handle_wheel_scroll, CanvasViewport};
pub use super::{BulkDragState, ClickRecord, ClickTarget, MarqueeState, CLICK_SLOP_PX};
use super::{CanvasEdge, CanvasNode, DiagramCanvas, DiagramCanvasState};
use crate::features::concept_model::{NodeId, GRID_SIZE};
use iced::mouse;
use iced::widget::canvas::{Action, Event, Frame};
use iced::{Rectangle, Vector};
use std::time::Instant;

fn publish_selection<Message>(
    on_selection_changed: &Option<Box<dyn Fn(std::collections::HashSet<NodeId>) -> Message + '_>>,
    on_node_selected: &(dyn Fn(Option<NodeId>) -> Message + '_),
    selection: std::collections::HashSet<NodeId>,
) -> Action<Message> {
    match on_selection_changed {
        Some(cb) => Action::publish(cb(selection)).and_capture(),
        None => Action::publish(on_node_selected(selection.iter().next().copied())).and_capture(),
    }
}

/// Central event-dispatcher for mus- og tastaturinteraktion på lærredet
pub fn handle_canvas_event<'a, Message, N, E, R>(
    canvas: &DiagramCanvas<'a, Message, N, E, R>,
    state: &mut DiagramCanvasState,
    event: &Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Option<Action<Message>>
where
    N: CanvasNode,
    E: CanvasEdge,
    R: Fn(&mut Frame, &N, bool, CanvasViewport) + 'a,
{
    if state.last_canvas_selection.as_ref() != Some(&canvas.selected_node_ids) {
        state.selected_node_ids = canvas.selected_node_ids.clone();
        state.last_canvas_selection = Some(canvas.selected_node_ids.clone());
    }

    let cursor_pos = cursor.position_in(bounds)?;

    match event {
        Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            state.modifiers = *modifiers;
            if matches!(
                key,
                iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete)
                    | iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace)
            ) {
                if let Some(action) = super::edge_interaction::handle_delete_bend_key(
                    state,
                    canvas.edges,
                    &canvas.on_edge_bends_changed,
                ) {
                    return Some(action);
                }
            }
            None
        }
        Event::Keyboard(iced::keyboard::Event::KeyReleased { modifiers, .. })
        | Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
            state.modifiers = *modifiers;
            None
        }
        Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
            let mut vp = canvas.viewport;
            handle_wheel_scroll(
                &mut vp,
                state.modifiers,
                *delta,
                cursor_pos,
                &canvas.on_viewport_changed,
            )
        }
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
            state.panning_start = Some((cursor_pos, canvas.viewport.pan()));
            Some(Action::capture())
        }
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
            if canvas.is_space_pressed {
                state.panning_start = Some((cursor_pos, canvas.viewport.pan()));
                state.is_panning_space = true;
                return Some(Action::capture());
            }

            state.panning_start = None;
            state.is_panning_space = false;

            // 0. Tjek kontrolpanel og miniaturekort (Minimap)
            let mut vp = canvas.viewport;
            if let Some(panel_action) = minimap::handle_panel_click(
                bounds,
                cursor_pos,
                canvas.nodes,
                &mut vp,
                &mut state.is_panning_minimap,
                &canvas.on_viewport_changed,
            ) {
                return Some(panel_action);
            }

            let world_pos = canvas.viewport.to_world(cursor_pos);

            // 1. Tjek forbindelseshåndtag
            if let Some(action) = super::edge_interaction::check_connect_handle_click(
                canvas.selected_node_id,
                canvas.nodes,
                state,
                world_pos,
            ) {
                return Some(action);
            }

            // 1.5. Tjek knækpunkthåndtag på valgt relation
            if let Some(action) = super::edge_interaction::check_bend_handle_click(
                canvas.selected_edge,
                canvas.edges,
                state,
                cursor_pos,
                world_pos,
                &canvas.on_edge_bends_changed,
            ) {
                return Some(action);
            }

            let is_ctrl = state.modifiers.control() || state.modifiers.command();
            state.press_screen_pos = Some(cursor_pos);
            let target = canvas
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
                let is_candidate = state
                    .last_click
                    .map(|last| {
                        let dist =
                            (cursor_pos.x - last.position.x).hypot(cursor_pos.y - last.position.y);
                        dist <= 20.0
                            && now.duration_since(last.time).as_millis() <= 500
                            && last.target == target
                    })
                    .unwrap_or(false);

                state.pending_double = if is_candidate {
                    let first_world_pos = state
                        .last_click
                        .map(|l| l.world_position)
                        .unwrap_or(world_pos);
                    Some((target, first_world_pos))
                } else {
                    None
                };
            }

            // 2. Klik på en node
            for node in canvas.nodes.iter().rev() {
                if node.contains(world_pos.x, world_pos.y) {
                    if is_ctrl {
                        if state.selected_node_ids.contains(&node.id()) {
                            state.selected_node_ids.remove(&node.id());
                        } else {
                            state.selected_node_ids.insert(node.id());
                        }
                        return Some(publish_selection(
                            &canvas.on_selection_changed,
                            &canvas.on_node_selected,
                            state.selected_node_ids.clone(),
                        ));
                    } else {
                        let is_already_selected = state.selected_node_ids.contains(&node.id())
                            || canvas.is_node_selected(node.id());
                        let drag_ids: Vec<NodeId> =
                            if is_already_selected && state.selected_node_ids.len() > 1 {
                                state.selected_node_ids.iter().copied().collect()
                            } else {
                                vec![node.id()]
                            };

                        let initial_positions: Vec<(NodeId, (f32, f32))> = canvas
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
                            start_time: Instant::now(),
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
                            return Some(publish_selection(
                                &canvas.on_selection_changed,
                                &canvas.on_node_selected,
                                state.selected_node_ids.clone(),
                            ));
                        } else {
                            return Some(Action::capture());
                        }
                    }
                }
            }

            // 3. Klik på en relation
            if let Some(action) = super::edge_interaction::check_edge_click(
                canvas.nodes,
                canvas.edges,
                canvas.selected_edge,
                state,
                cursor_pos,
                world_pos,
                &canvas.on_edge_selected,
            ) {
                return Some(action);
            }

            // 4. Klik på tomt lærred
            state.selected_bend = None;
            if let Some(ref on_edge_selected) = canvas.on_edge_selected {
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

            let world_pos = canvas.viewport.to_world(cursor_pos);
            if let Some(action) =
                super::edge_interaction::handle_bend_or_segment_cursor_drag(state, world_pos)
            {
                return Some(action);
            }

            if state.is_panning_minimap {
                let mut vp = canvas.viewport;
                let action = minimap::handle_minimap_drag(
                    bounds,
                    cursor_pos,
                    canvas.nodes,
                    &mut vp,
                    &canvas.on_viewport_changed,
                );
                return Some(action);
            }

            if let Some((start_pos, initial_pan)) = state.panning_start {
                let total_delta = cursor_pos - start_pos;
                let mut new_vp = canvas.viewport;
                new_vp.set_pan(initial_pan + total_delta);
                return Some(Action::publish((canvas.on_viewport_changed)(new_vp)).and_capture());
            }

            if let Some(action) = super::edge_interaction::handle_connecting_cursor_move(
                state,
                canvas.nodes,
                world_pos,
            ) {
                return Some(action);
            }

            if let Some(ref mut marquee) = state.marquee {
                let world_pos = canvas.viewport.to_world(cursor_pos);
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

                let world_pos = canvas.viewport.to_world(cursor_pos);
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

                if bulk.last_delta == Some((delta_x, delta_y)) {
                    return Some(Action::capture());
                }
                bulk.last_delta = Some((delta_x, delta_y));

                let updates: Vec<(NodeId, f32, f32)> = bulk
                    .initial_positions
                    .iter()
                    .map(|&(id, (ix, iy))| (id, ix + delta_x, iy + delta_y))
                    .collect();

                if let Some(ref on_nodes_moved) = canvas.on_nodes_moved {
                    return Some(Action::publish((on_nodes_moved)(updates)).and_capture());
                } else if let Some(&(lid, lx, ly)) =
                    updates.iter().find(|(id, _, _)| *id == bulk.leader_id)
                {
                    return Some(
                        Action::publish((canvas.on_node_moved)(lid, lx, ly)).and_capture(),
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

                let world_pos = canvas.viewport.to_world(cursor_pos);
                let raw_world_x = world_pos.x - offset.x;
                let raw_world_y = world_pos.y - offset.y;
                let new_x = (raw_world_x / GRID_SIZE).round() * GRID_SIZE;
                let new_y = (raw_world_y / GRID_SIZE).round() * GRID_SIZE;

                if let Some(node) = canvas.nodes.iter().find(|n| n.id() == id) {
                    let (cx, cy) = node.position();
                    if (cx - new_x).abs() < f32::EPSILON && (cy - new_y).abs() < f32::EPSILON {
                        return Some(Action::capture());
                    }
                }

                return Some(
                    Action::publish((canvas.on_node_moved)(id, new_x, new_y)).and_capture(),
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

            if let Some(action) =
                super::edge_interaction::handle_connecting_release(state, &canvas.on_edge_created)
            {
                return Some(action);
            }

            if let Some(action) = super::edge_interaction::handle_bend_or_segment_release(
                state,
                cursor_pos,
                &canvas.on_edge_bends_changed,
            ) {
                return Some(action);
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
                                Action::publish((canvas.on_node_double_clicked)(id)).and_capture(),
                            );
                        }
                        ClickTarget::Bend((from, to), idx) => {
                            return super::edge_interaction::handle_bend_double_click(
                                from,
                                to,
                                idx,
                                canvas.edges,
                                state,
                                &canvas.on_edge_bends_changed,
                            );
                        }
                        ClickTarget::Empty => {
                            return Some(
                                Action::publish((canvas.on_canvas_double_clicked)(
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
                    if !state.modifiers.control() && !state.modifiers.command() {
                        state.last_click = Some(ClickRecord {
                            position: cursor_pos,
                            world_position: canvas.viewport.to_world(cursor_pos),
                            time: Instant::now(),
                            target: ClickTarget::Empty,
                        });
                        state.selected_node_ids.clear();
                        if let Some(ref on_edge_selected) = canvas.on_edge_selected {
                            let _ = (on_edge_selected)(None);
                        }
                        return Some(publish_selection(
                            &canvas.on_selection_changed,
                            &canvas.on_node_selected,
                            std::collections::HashSet::new(),
                        ));
                    }
                } else {
                    state.last_click = None;
                    let selected_in_box = marquee.select_nodes(canvas.nodes);
                    let new_selection = if state.modifiers.control() || state.modifiers.command() {
                        let mut combined = state.selected_node_ids.clone();
                        combined.extend(selected_in_box);
                        combined
                    } else {
                        selected_in_box
                    };
                    state.selected_node_ids = new_selection.clone();
                    return Some(publish_selection(
                        &canvas.on_selection_changed,
                        &canvas.on_node_selected,
                        new_selection,
                    ));
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
                    if let Some(ref on_drag_released) = canvas.on_drag_released {
                        return Some(Action::publish((on_drag_released)()).and_capture());
                    }
                } else {
                    if !state.modifiers.control() && !state.modifiers.command() {
                        state.last_click = Some(ClickRecord {
                            position: cursor_pos,
                            world_position: canvas.viewport.to_world(cursor_pos),
                            time: Instant::now(),
                            target: ClickTarget::Node(bulk.leader_id),
                        });
                    }
                    if state.selected_node_ids.len() > 1
                        && !state.modifiers.control()
                        && !state.modifiers.command()
                    {
                        state.selected_node_ids = [bulk.leader_id].into_iter().collect();
                        return Some(publish_selection(
                            &canvas.on_selection_changed,
                            &canvas.on_node_selected,
                            state.selected_node_ids.clone(),
                        ));
                    }
                }
                return Some(Action::capture());
            }

            if state.dragging_node.is_some() {
                state.dragging_node = None;
                let is_drag = move_dist >= CLICK_SLOP_PX;
                if is_drag {
                    if let Some(ref on_drag_released) = canvas.on_drag_released {
                        return Some(Action::publish((on_drag_released)()).and_capture());
                    }
                }
                return Some(Action::capture());
            }

            if canvas.is_palette_dragging {
                if let Some(ref on_canvas_drop) = canvas.on_canvas_drop {
                    let world_pos = canvas.viewport.to_world(cursor_pos);
                    return Some(
                        Action::publish((on_canvas_drop)(world_pos.x, world_pos.y)).and_capture(),
                    );
                }
            }

            None
        }
        _ => None,
    }
}
