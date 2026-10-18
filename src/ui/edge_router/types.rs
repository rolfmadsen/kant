pub use crate::features::concept_model::PortSide;
use crate::features::concept_model::{NodeId, RelationKind};
use iced::Point;
use std::collections::HashMap;

/// Minimum clearance in pixels needed between node borders to draw direct facing arrows.
pub const MIN_ARROW_CLEARANCE: f32 = 36.0;
/// Length of the UML generalization arrowhead triangle.
pub const ARROW_HEAD_LENGTH: f32 = 14.0;
/// Base width of the UML generalization arrowhead triangle.
pub const ARROW_HEAD_WIDTH: f32 = 14.0;
/// Length of the UML composition diamond from tip to back.
pub const DIAMOND_LENGTH: f32 = 14.0;
/// Width of the UML composition diamond between lateral vertices.
pub const DIAMOND_WIDTH: f32 = 14.0;
/// Horizontal/vertical spacing between multiple relation ports on the same node side.
pub const SLOT_SPACING: f32 = 24.0;
/// Channel offset between parallel orthogonal line segments.
pub const CHANNEL_OFFSET: f32 = 14.0;
/// Radius for line jump bridges over intersecting edges.
pub const BRIDGE_RADIUS: f32 = 5.0;

/// Represents the geometric triangle of a UML arrowhead touching a node boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrowHead {
    pub tip: Point,
    pub left: Point,
    pub right: Point,
    pub direction: PortSide,
}

/// Represents the geometric diamond of a UML composition touching the source node boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct Diamond {
    pub tip: Point,
    pub left: Point,
    pub right: Point,
    pub back: Point,
    pub direction: PortSide,
}

/// Represents a half arrow (single diagonal barb) for UML directed association.
#[derive(Debug, Clone, PartialEq)]
pub struct HalfArrow {
    pub tip: Point,
    pub barb: Point,
    pub direction: PortSide,
}

/// A visual bridge (line jump) rendered where a horizontal segment crosses a vertical segment.
#[derive(Debug, Clone, PartialEq)]
pub struct BridgeHop {
    pub center: Point,
    pub is_horizontal: bool,
}

/// Fully computed 90-degree orthogonal route for a diagram edge.
#[derive(Debug, Clone, PartialEq)]
pub struct RoutedEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: RelationKind,
    pub label: Option<String>,
    pub label_pos: Option<Point>,
    pub points: Vec<Point>,
    pub arrow_head: Option<ArrowHead>,
    pub half_arrow: Option<HalfArrow>,
    pub source_diamond: Option<Diamond>,
    pub bridges: Vec<BridgeHop>,
    pub from_side: PortSide,
    pub to_side: PortSide,
}

pub type SideAttachment = (usize, RelationKind, bool);
pub type SideAttachmentMap = HashMap<(NodeId, PortSide), Vec<SideAttachment>>;

#[derive(Debug, Clone)]
pub struct EdgePortAssignment {
    pub edge_idx: usize,
    pub from_id: NodeId,
    pub to_id: NodeId,
    pub kind: RelationKind,
    pub from_side: PortSide,
    pub to_side: PortSide,
    pub from_slot_offset: f32,
    pub to_slot_offset: f32,
    pub channel_index: usize,
}
