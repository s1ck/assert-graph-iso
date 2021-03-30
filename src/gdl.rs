use gdl::CypherValue;

use crate::graph::{Graph, LabelIterator, NodesIterator, PropertyIterator};

impl Graph for gdl::Graph {
    type NodeId = str;

    type NodeLabel = str;

    type RelationshipType = str;

    type PropertyKey = str;

    type PropertyValue = CypherValue;

    fn nodes(&self) -> NodesIterator<&Self::NodeId> {
        Box::new(self.nodes().map(|node| node.variable()))
    }

    fn node_labels(&self, node_id: &Self::NodeId) -> LabelIterator<&Self::NodeLabel> {
        let node = self
            .get_node(node_id)
            .unwrap_or_else(|| panic!("Node id {} not found", node_id));
        Box::new(node.labels())
    }

    fn node_properties(
        &self,
        node_id: &Self::NodeId,
    ) -> PropertyIterator<&Self::PropertyKey, &Self::PropertyValue> {
        let node = self
            .get_node(node_id)
            .unwrap_or_else(|| panic!("Node id {} not found", node_id));
        Box::new(node.properties())
    }

    fn outgoing_relationships<'a, 'b: 'a>(
        &'a self,
        node_id: &'b Self::NodeId,
    ) -> PropertyIterator<
        'a,
        (&'a Self::NodeId, &'a Self::RelationshipType),
        PropertyIterator<'a, &'a Self::PropertyKey, &'a Self::PropertyValue>,
    > {
        Box::new(self.relationships().filter_map(move |rel| {
            (rel.source() == node_id).then(|| {
                let key = (rel.target(), rel.rel_type().unwrap_or(""));
                let value: Box<dyn Iterator<Item = (&str, &CypherValue)>> =
                    Box::new(rel.properties());
                (key, value)
            })
        }))
    }

    fn incoming_relationships<'a, 'b: 'a>(
        &'a self,
        node_id: &'b Self::NodeId,
    ) -> PropertyIterator<
        'a,
        (&'a Self::NodeId, &'a Self::RelationshipType),
        PropertyIterator<'a, &'a Self::PropertyKey, &'a Self::PropertyValue>,
    > {
        Box::new(self.relationships().filter_map(move |rel| {
            (rel.target() == node_id).then(|| {
                let key = (rel.source(), rel.rel_type().unwrap_or(""));
                let value: Box<dyn Iterator<Item = (&str, &CypherValue)>> =
                    Box::new(rel.properties());
                (key, value)
            })
        }))
    }
}
