use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
};

use gdl::CypherValue;

type NodesIterator<'a, T> = Box<dyn Iterator<Item = T> + 'a>;
type LabelIterator<'a, T> = Box<dyn Iterator<Item = T> + 'a>;
type PropertyIterator<'a, K, V> = Box<dyn Iterator<Item = (K, V)> + 'a>;

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

pub fn canonicalize<G: Graph>(graph: &G) -> String {
    let canonical_nodes = canonical_nodes(graph);

    dbg!(&canonical_nodes);

    let mut out_adjacencies = HashMap::<&G::NodeId, Vec<String>>::new();
    let mut in_adjacencies = HashMap::<&G::NodeId, Vec<String>>::new();

    graph.nodes().for_each(|source_node| {
        graph.outgoing_relationships(source_node).for_each(
            |((target_node, rel_type), rel_properties)| {
                let canonical_source = canonical_nodes.get(source_node).unwrap();
                let canonical_target = canonical_nodes.get(target_node).unwrap();

                let sorted_properties = canonical_properties::<G>(rel_properties);

                let canonical_out_relationship = format!(
                    "()-[:{} {}]->{}",
                    rel_type, sorted_properties, canonical_target
                );

                let canonical_in_relationship = format!(
                    "()<-[:{} {}]-{}",
                    rel_type, sorted_properties, canonical_source
                );

                out_adjacencies
                    .entry(source_node)
                    .or_insert(Vec::new())
                    .push(canonical_out_relationship);

                in_adjacencies
                    .entry(target_node)
                    .or_insert(Vec::new())
                    .push(canonical_in_relationship);
            },
        )
    });

    dbg!(&out_adjacencies);
    dbg!(&in_adjacencies);

    let mut canonical_out_adjacencies = out_adjacencies
        .into_iter()
        .map(|(node, mut relationships)| {
            relationships.sort();
            (node, relationships.join(", "))
        })
        .collect::<HashMap<_, _>>();

    let mut canonical_in_adjacencies = in_adjacencies
        .into_iter()
        .map(|(node, mut relationships)| {
            relationships.sort();
            (node, relationships.join(", "))
        })
        .collect::<HashMap<_, _>>();

    dbg!(&canonical_out_adjacencies);
    dbg!(&canonical_in_adjacencies);

    let mut matrix = canonical_nodes
        .into_iter()
        .map(|(node, canonical_node)| {
            format!(
                "{} => out: {} in: {}",
                canonical_node,
                canonical_out_adjacencies.remove(node).unwrap_or_default(),
                canonical_in_adjacencies.remove(node).unwrap_or_default()
            )
        })
        .collect::<Vec<_>>();

    matrix.sort();
    matrix.join("\n")
}

fn canonical_nodes<G: Graph>(graph: &G) -> HashMap<&G::NodeId, String> {
    graph
        .nodes()
        .map(|node| {
            let mut node_labels = graph
                .node_labels(node)
                .map(|label| format!("{}", label))
                .collect::<Vec<_>>();

            node_labels.sort();
            node_labels.dedup();

            let sorted_labels = node_labels
                .into_iter()
                .map(|label| format!(":{}", label))
                .collect::<String>();

            let sorted_properties = canonical_properties::<G>(graph.node_properties(node));

            (node, format!("({} {})", sorted_labels, sorted_properties))
        })
        .collect::<HashMap<_, _>>()
}

fn canonical_properties<G: Graph>(
    properties: PropertyIterator<&G::PropertyKey, &G::PropertyValue>,
) -> String {
    let mut properties = properties
        .map(|(key, value)| format!("{}: {}", key, value))
        .collect::<Vec<_>>();

    properties.dedup();
    properties.sort();

    let sorted_properties = properties.join(", ");
    if sorted_properties.is_empty() {
        String::new()
    } else {
        format!("{{ {} }}", sorted_properties)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trim_margin::MarginTrimmable;

    #[test]
    fn canonical_labels() {
        let g = gdl::Graph::from(
            r#"
              (a:A { c: 42, b: 37, a: 13 })
            , (b:B { bar: 84 })
            , (c:C { baz: 19, boz: 84 })
            , (a)-[:REL { c: 42, b: 37, a: 13 }]->(b)
            , (b)-[:REL { c: 12 }]->(a)
            , (b)-[:REL { a: 23 }]->(c)
            "#,
        )
        .unwrap();

        let expected = "
            |(:A { a: 13, b: 37, c: 42 }) => out: ()-[:REL { a: 13, b: 37, c: 42 }]->(:B { bar: 84 }) in: ()<-[:REL { c: 12 }]-(:B { bar: 84 })
            |(:B { bar: 84 }) => out: ()-[:REL { a: 23 }]->(:C { baz: 19, boz: 84 }), ()-[:REL { c: 12 }]->(:A { a: 13, b: 37, c: 42 }) in: ()<-[:REL { a: 13, b: 37, c: 42 }]-(:A { a: 13, b: 37, c: 42 })
            |(:C { baz: 19, boz: 84 }) => out:  in: ()<-[:REL { a: 23 }]-(:B { bar: 84 })
            ".trim_margin().unwrap();

        assert_eq!(expected, canonicalize(&g));
    }
}
