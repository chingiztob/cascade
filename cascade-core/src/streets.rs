use std::path::Path;

use ahash::{HashMap, HashMapExt};
use osm4routing;
use petgraph::graph::DiGraph;
use rustworkx_core::connectivity::{connected_components, number_connected_components};

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

        graph.add_edge(source_index, target_index, edge_type.clone());
        // TODO: Filter duplicate edges
        graph.add_edge(target_index, source_index, edge_type);
    }
    
    let largest_component = connected_components(&graph)
        .iter()
        .max_by_key(|set| set.len())
        .cloned()
        .unwrap();

    // Create a new graph for the largest component
    let mut new_graph = DiGraph::<GraphNode, GraphEdge>::new();
    let mut new_node_indices = HashMap::new();

    // Add nodes from the largest component to the new graph
    for &node_index in &largest_component {
        let node = graph[node_index].clone(); // Clone the node
        let new_node_index = new_graph.add_node(node); // Add to new graph
        new_node_indices.insert(node_index, new_node_index); // Keep track of indices
    }

    // Add edges from the largest component to the new graph
    for &node_index in &largest_component {
        for neighbor in graph.neighbors(node_index) {
            let edge = graph.find_edge(node_index, neighbor).unwrap(); // Get the edge
            let edge_type = graph[edge].clone(); // Clone the edge type

            // Add edge to the new graph using the new node indices
            new_graph.add_edge(
                new_node_indices[&node_index],
                new_node_indices[&neighbor],
                edge_type,
            );
        }
    }
    println!("{}", number_connected_components(&new_graph));
    println!("{}", new_graph.node_count());

    let tree = build_rtree(&new_graph);
    let graph = TransitGraph::from_parts(new_graph, tree);

    Ok(graph)
}
