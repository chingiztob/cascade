use std::path::Path;

use ahash::{HashMap, HashMapExt};
use osm4routing;
use petgraph::graph::DiGraph;

use crate::connectors::build_rtree;
use crate::graph::{GraphEdge, GraphNode, TransitGraph, WalkEdge, WalkNode};
use crate::Error;

pub(crate) fn create_graph(filename: impl AsRef<Path>) -> Result<TransitGraph, Error> {
    let mut graph = DiGraph::<GraphNode, GraphEdge>::new();
    // This hashmap is used to store OSM node IDs and their corresponding graph node indices
    // This is required to avoid creating duplicate nodes for the same OSM node ID
    // Unfortunately, petgraph `graph` does not provide a method to check if a node exists
    let (nodes, edges) = osm4routing::Reader::new()
        .read_tag("highway")
        .read(&filename)
        .map_err(|e| Error::InvalidData(format!("Error reading PBF: {e}")))?;

    let mut node_indices = HashMap::new();

    for node in nodes {
        node_indices.entry(node.id).or_insert_with(|| {
            let node = GraphNode::Walk(WalkNode {
                id: node.id,
                geometry: node.coord.into(),
            });

            graph.add_node(node)
        });
    }

    for edge in edges {
        let source_index = *node_indices
            .get(&edge.source)
            .ok_or_else(|| Error::MissingKey(edge.source))?;
        let target_index = *node_indices
            .get(&edge.target)
            .ok_or_else(|| Error::MissingKey(edge.target))?;

        let edge_type = GraphEdge::Walk(WalkEdge {
            edge_weight: edge.length(),
        });

        graph.add_edge(source_index, target_index, edge_type);
    }

    let tree = build_rtree(&graph);
    let graph = TransitGraph::from_parts(graph, tree);

    Ok(graph)
}
