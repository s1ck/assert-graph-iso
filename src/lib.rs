use std::collections::HashMap;

use graph::PropertyIterator;

#[cfg(feature = "gdl")]
pub mod gdl;
pub mod graph;

pub use graph::Graph;

pub fn assert_graph_eq(left: &impl Graph, right: &impl Graph) -> bool {
    let left = canonicalize(left);
    let right = canonicalize(right);
    left.eq(&right)
}

fn canonicalize<G: Graph>(graph: &G) -> String {
    let canonical_nodes = canonical_nodes(graph);

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

    &canonical_out_adjacencies;
    &canonical_in_adjacencies;

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

#[cfg(all(not(feature = "gdl"), test))]
compile_error!("Please run tests with --all-features");

#[cfg(all(feature = "gdl", test))]
mod tests {
    use super::*;

    use ::gdl::Graph as GdlGraph;
    use trim_margin::MarginTrimmable;

    fn from_gdl(gdl: &str) -> GdlGraph {
        gdl.parse::<GdlGraph>().unwrap()
    }

    #[test]
    fn test_topology_equals() {
        let g1 = from_gdl("(a), (b), (a)-->(b)");
        let g2 = from_gdl("(a), (b), (a)-->(b)");

        assert_eq!(canonicalize(&g1), canonicalize(&g2))
    }

    #[test]
    fn test_topology_not_equals() {
        let g1 = from_gdl("(a), (b), (a)-->(b)");
        let g2 = from_gdl("(a), (a)-->(a)");
        assert_ne!(canonicalize(&g1), canonicalize(&g2))
    }

    #[test]
    fn test_topology_and_node_labels_equals() {
        let g1 = from_gdl("(a:A:B), (b:B), (a)-->(b)");
        let g2 = from_gdl("(a:A:B), (b:B), (a)-->(b)");
        assert_eq!(canonicalize(&g1), canonicalize(&g2))
    }

    #[test]
    fn test_topology_and_node_labels_not_equals() {
        let g1 = from_gdl("(a:A:B), (b:B), (a)-->(b)");
        let g2 = from_gdl("(a:A:B), (b:C), (a)-->(b)");
        assert_ne!(canonicalize(&g1), canonicalize(&g2))
    }

    #[test]
    fn test_topology_and_data_equals() {
        let g1 = from_gdl("(a {a:2, w:1.0}), (b {w:2, a:3, q:42.0}), (a)-->(b)");
        let g2 = from_gdl("(a {a:2, w:1.0}), (b {w:2, a:3, q:42.0}), (a)-->(b)");
        assert_eq!(canonicalize(&g1), canonicalize(&g2))
    }

    #[test]
    fn test_parallel_edges() {
        let g1 = from_gdl("(a), (b), (a)-[{w:1}]->(b), (a)-[{w:2}]->(b)");
        let g2 = from_gdl("(a), (b), (a)-[{w:2}]->(b), (a)-[{w:1}]->(b)");
        assert_eq!(canonicalize(&g1), canonicalize(&g2))
    }

    #[test]
    fn test_loop() {
        let g1 = from_gdl("(a), (b), (a)-[{w:1}]->(a), (a)-[{w:2}]->(b)");
        let g2 = from_gdl("(a), (b), (a)-[{w:2}]->(b), (a)-[{w:1}]->(a)");
        assert_eq!(canonicalize(&g1), canonicalize(&g2))
    }

    #[test]
    fn test_cycle() {
        let g1 = from_gdl("(a {v:1}), (b {v:2}), (c {v:3}), (a)-->(b)-->(c)-->(a)");
        let g2 = from_gdl("(a {v:2}), (b {v:3}), (c {v:1}), (a)-->(b)-->(c)-->(a)");
        assert_eq!(canonicalize(&g1), canonicalize(&g2))
    }

    #[test]
    fn test_complete_graph() {
        let g1 = from_gdl(
            "(a {v:1}), (b {v:2}), (c {v:3}), (b)<--(a)-->(c), (a)<--(b)-->(c), (a)<--(c)-->(b)",
        );
        let g2 = from_gdl(
            "(a {v:1}), (b {v:2}), (c {v:3}), (b)<--(a)-->(b), (a)<--(b)-->(c), (a)<--(c)-->(b)",
        );
        assert_ne!(canonicalize(&g1), canonicalize(&g2))
    }

    #[test]
    fn test_complete_homogenic_graph() {
        let g1 = from_gdl(
            "(a {v:1}), (b {v:1}), (c {v:1}), (b)<--(a)-->(c), (a)<--(b)-->(c), (a)<--(c)-->(b)",
        );
        let g2 = from_gdl(
            "(a {v:1}), (b {v:1}), (c {v:1}), (b)<--(a)-->(b), (a)<--(b)-->(c), (a)<--(c)-->(b)",
        );
        assert_ne!(canonicalize(&g1), canonicalize(&g2))
    }

    #[test]
    fn test_canonicalize() {
        let g = r#"
              (a:A { c: 42, b: 37, a: 13 })
            , (b:B { bar: 84 })
            , (c:C { baz: 19, boz: 84 })
            , (a)-[:REL { c: 42, b: 37, a: 13 }]->(b)
            , (b)-[:REL { c: 12 }]->(a)
            , (b)-[:REL { a: 23 }]->(c)
            "#
        .parse::<GdlGraph>()
        .unwrap();

        let expected = "
            |(:A { a: 13, b: 37, c: 42 }) => out: ()-[:REL { a: 13, b: 37, c: 42 }]->(:B { bar: 84 }) in: ()<-[:REL { c: 12 }]-(:B { bar: 84 })
            |(:B { bar: 84 }) => out: ()-[:REL { a: 23 }]->(:C { baz: 19, boz: 84 }), ()-[:REL { c: 12 }]->(:A { a: 13, b: 37, c: 42 }) in: ()<-[:REL { a: 13, b: 37, c: 42 }]-(:A { a: 13, b: 37, c: 42 })
            |(:C { baz: 19, boz: 84 }) => out:  in: ()<-[:REL { a: 23 }]-(:B { bar: 84 })
            ".trim_margin().unwrap();

        assert_eq!(expected, canonicalize(&g));
    }
}
