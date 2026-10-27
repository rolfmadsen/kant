use crate::features::concept_model::{
    BendPoint, DiagramEdge, DiagramNode, NodeId, PortSide, RelationKind,
};
use crate::features::information_model::{ClassDiagramEdge, ClassDiagramNode};

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
    fn bends(&self) -> Option<&[BendPoint]> {
        None
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
        if let Some(b) = self.bends() {
            edge.set_bends(Some(b.to_vec()));
        }
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
    fn bends(&self) -> Option<&[BendPoint]> {
        self.bends()
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
    fn bends(&self) -> Option<&[BendPoint]> {
        self.bends()
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
