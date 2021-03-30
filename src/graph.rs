use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

pub type NodesIterator<'a, T> = Box<dyn Iterator<Item = T> + 'a>;
pub type LabelIterator<'a, T> = Box<dyn Iterator<Item = T> + 'a>;
pub type PropertyIterator<'a, K, V> = Box<dyn Iterator<Item = (K, V)> + 'a>;

pub trait Graph {
    type NodeId: Debug + Hash + Eq + ?Sized;

    type NodeLabel: Display + ?Sized;

    type RelationshipType: Display + ?Sized;

    type PropertyKey: Display + ?Sized;

    type PropertyValue: Display + ?Sized;

    fn nodes(&self) -> NodesIterator<&Self::NodeId>;

    fn node_labels(&self, node_id: &Self::NodeId) -> LabelIterator<&Self::NodeLabel>;

    fn node_properties(
        &self,
        node_id: &Self::NodeId,
    ) -> PropertyIterator<&Self::PropertyKey, &Self::PropertyValue>;

    fn outgoing_relationships<'a, 'b: 'a>(
        &'a self,
        node_id: &'b Self::NodeId,
    ) -> PropertyIterator<
        (&'a Self::NodeId, &'a Self::RelationshipType),
        PropertyIterator<&'a Self::PropertyKey, &'a Self::PropertyValue>,
    >;

    fn incoming_relationships<'a, 'b: 'a>(
        &'a self,
        node_id: &'b Self::NodeId,
    ) -> PropertyIterator<
        (&'a Self::NodeId, &'a Self::RelationshipType),
        PropertyIterator<&'a Self::PropertyKey, &'a Self::PropertyValue>,
    >;
}
