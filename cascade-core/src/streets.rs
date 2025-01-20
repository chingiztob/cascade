use std::path::Path;

use geo::LineString;
use hashbrown::HashMap;
use osm4routing::{self, FootAccessibility};
use petgraph::graph::DiGraph;
use rustworkx_core::connectivity::connected_components;

use crate::connectors::build_rtree;
use crate::graph::{GraphEdge, GraphNode, TransitGraph, WalkEdge, WalkNode};
use crate::Error;

/// Creates a transit graph from an OpenStreetMap (OSM) PBF file.
///
/// This function reads OSM data, filters it by the "highway" tag, and constructs a directed graph
/// (`DiGraph`) where nodes represent OSM nodes and edges represent OSM ways.
///
/// The function then identifies the largest connected component in the graph and creates a new
/// graph containing only this component. This new graph is used to build an R-tree for spatial
/// indexing.
#[allow(clippy::redundant_closure_for_method_calls)]
pub(crate) fn create_graph(filename: impl AsRef<Path>) -> Result<TransitGraph, Error> {
    let mut graph = DiGraph::<GraphNode, GraphEdge>::new();
    // This hashmap is used to store OSM node IDs and their corresponding graph node indices
    // This is required to avoid creating duplicate nodes for the same OSM node ID
    // Unfortunately, petgraph `graph` does not provide a method to check if a node exists
    let (nodes, edges) = osm4routing::Reader::new()
        .merge_ways()
        .read_tag("highway")
        .read(&filename)
        .map_err(|e| Error::InvalidData(format!("Error reading PBF: {e}")))?;

    // filter only pedestrian allowed ways
    let edges = edges
        .into_iter()
        .filter(|edge| {
            matches!(
                edge.properties.foot,
                FootAccessibility::Allowed | FootAccessibility::Unknown
            )
        })
        .collect::<Vec<_>>();

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
            .ok_or(Error::MissingKey(edge.source))?;
        let target_index = *node_indices
            .get(&edge.target)
            .ok_or(Error::MissingKey(edge.target))?;

        let edge_weight = edge.length();
        let geometry: Option<LineString<f64>> = Some(edge.geometry.into());

        let edge_type = GraphEdge::Walk(WalkEdge {
            edge_weight,
            geometry,
        });

        graph.add_edge(source_index, target_index, edge_type.clone());
        graph.add_edge(target_index, source_index, edge_type);
    }

    let largest_component = connected_components(&graph)
        .into_iter()
        .max_by_key(|set| set.len())
        .ok_or(Error::MissingValue(
            "No connected components found".to_string(),
        ))?;

    // Create a new graph for the largest connected component
    let mut new_graph = DiGraph::<GraphNode, GraphEdge>::new();
    let mut new_node_indices = HashMap::new();

    for node_index in &largest_component {
        let node = graph[*node_index].clone();
        let new_node_index = new_graph.add_node(node);
        new_node_indices.insert(node_index, new_node_index); // Keep track of indices
    }

    for node_index in &largest_component {
        for neighbor in graph.neighbors(*node_index) {
            let edge = graph.find_edge(*node_index, neighbor).unwrap();
            let edge_type = graph[edge].clone();

            new_graph.add_edge(
                new_node_indices[&node_index],
                new_node_indices[&neighbor],
                edge_type,
            );
        }
    }

    let tree = build_rtree(&new_graph);
    let graph = TransitGraph::from_parts(new_graph, tree);

    Ok(graph)
}
