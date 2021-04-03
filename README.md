### assert-graph-iso

A test utility to check if two property graphs are equal, i.e., isomorphic.
The check is performed by computing a canonical string representation for each graph.
If the canonical representations are identical, the graphs are considered isomorphic.
The crate is supposed to be used as a test utility, it is not designed for large scale graph comparisons.


#### Property graph data model

A property graph consists of nodes and relationships.
Nodes have zero or more labels, relationships have zero or one relationship type.
Both, nodes and relationships have properties, organized as key-value-pairs.
Relationships are directed, starting at a source node and pointing at a target node.


#### Usage

The crate contains a `Graph` trait which defines a property graph.
Users are supposed to implement the trait for their custom graph implemention.
The crate also provides a `gdl` feature which allows for simple graph definition using a declarative language.
Check out the [gdl on crates.io](https://crates.io/crates/gdl) for more information about the language.

Testing for equality:

```rust
use ::gdl::Graph as GdlGraph;
use assert_graph_iso::*;

let g1 = "(a), (b), (a)-[:REL { foo:42 }]->(b)".parse::<GdlGraph>().unwrap();
let g2 = "(a), (b), (b)-[:REL { foo:42 }]->(a)".parse::<GdlGraph>().unwrap();

assert!(equals(&g1, &g2))
```

Compare the canonical representations for easier debugging:

```rust
use ::gdl::Graph as GdlGraph;
use assert_graph_iso::*;

let g1 = "(a:Label1), (b:Label2), (a)-->(b)".parse::<GdlGraph>().unwrap();
let g2 = "(a:Label2), (b:Label1), (b)-->(a)".parse::<GdlGraph>().unwrap();

assert_eq!(canonicalize(&g1), canonicalize(&g2))
```


### License

Apache 2.0 or MIT
