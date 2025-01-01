use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::BinaryHeap;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use geo::{line_string, Coord};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;

use crate::algo::itinerary::segment::{Itinerary, Segment};
use crate::algo::MinScored;
use crate::graph::TransitGraph;
use crate::prelude::SnappedPoint;

/// Finds the shortest paths from a source node to a target node in a time-dependent graph
/// using a variation of Dijkstra's algorithm.
///
/// # Arguments
///
/// * `graph` - A reference to a `TransitGraph` object, representing the transit network.
/// * `start` - The source node index within the graph (`NodeIndex`).
/// * `target` - The target node index within the graph (`NodeIndex`).
/// * `start_time` - The starting time in seconds since midnight.
///
/// # Returns
///
/// An `Itinerary` object, representing the computed travel path.
///
/// The `Itinerary` contains the sequence of segments representing transitions along edges
/// in the graph and associated properties such as travel times and geometric details
/// (e.g., `LineString` geometries for each segment). If no valid path exists between the
/// source and target nodes, an empty `Itinerary` is returned.
///
/// # Algorithm Notes
///
/// - This implementation uses a priority queue (`BinaryHeap`) to perform a cost-based search.
/// - The time-dependent properties of edges are computed via `Segment::calculate_itinerary()`,
///   which adjusts travel weights based on the current traversal time.
/// - The algorithm tracks the shortest distances in a `HashMap` (`scores`) and stores predecessors
///   to reconstruct the final path.
/// - Geometric information about the path, such as linestrings, is included by deriving it from
///   the positions of connected nodes.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn detailed_itinerary_internal(
    graph: &TransitGraph,
    start: NodeIndex,
    target: NodeIndex,
    start_time: u32,
) -> Itinerary {
    let mut visited = HashSet::new();
    let mut scores: HashMap<NodeIndex, f64> =
        HashMap::with_capacity(graph.into_inner_graph().node_count());

    let mut visit_next = BinaryHeap::new();
    scores.insert(start, 0.0);
    visit_next.push(MinScored(0.0, (start, start_time)));

    let mut predecessors: HashMap<NodeIndex, (NodeIndex, Segment)> = HashMap::new();

    while let Some(MinScored(node_score, (node, current_time))) = visit_next.pop() {
        if visited.contains(&node) {
            continue;
        }

        if node == target {
            break;
        }

        for edge in graph.edges(node) {
            let current_node = edge.source();
            let next_node = edge.target();

            let edge_geometry = line_string![
                Coord::from(*graph.node_weight(current_node).unwrap().geometry()),
                Coord::from(*graph.node_weight(next_node).unwrap().geometry())
            ];

            if visited.contains(&next_node) {
                continue;
            }

            let segment = edge
                .weight()
                .calculate_itinerary(current_time, edge_geometry);

            if matches!(segment, Segment::NoService) {
                continue;
            }

            let travel_time = segment.weight();

            let next_score = node_score + travel_time;
            let next_time = current_time + travel_time as u32;

            match scores.entry(next_node) {
                Occupied(mut ent) => {
                    if next_score < *ent.get() {
                        ent.insert(next_score);
                        visit_next.push(MinScored(next_score, (next_node, next_time)));
                        predecessors.insert(next_node, (node, segment));
                    }
                }
                Vacant(ent) => {
                    ent.insert(next_score);
                    visit_next.push(MinScored(next_score, (next_node, next_time)));
                    predecessors.insert(next_node, (node, segment));
                }
            }
        }
        visited.insert(node);
    }

    // Reconstruct the path

    if scores.contains_key(&target) {
        let mut itinerary = Itinerary::new();
        let mut current_node = target;

        while let Some((prev_node, segment)) = predecessors.get(&current_node) {
            itinerary.push(segment.clone());
            current_node = *prev_node;
        }

        itinerary.travel.reverse();
        return itinerary;
    }

    Itinerary::new()
}

/// Computes an itinerary between two locations in a transit graph.
///
/// # Arguments
///
/// * `graph` - A reference to a `TransitGraph` object, representing the transit network.
/// * `start` - The starting point, represented as a `SnappedPoint` (snap-matched to a graph node).
/// * `target` - The target point, represented as a `SnappedPoint` (snap-matched to a graph node).
/// * `start_time` - The starting time in seconds since midnight.
///
/// # Returns
///
/// An `Itinerary` object containing the shortest path from `start` to `target`.
///
/// - Each segment within the `Itinerary` encapsulates detailed travel information,
///   including duration, geometry (e.g., linestrings), and transit characteristics.
/// - If no path is found, the returned itinerary will be empty.
///
pub fn detailed_itinerary<'a, 'b>(
    graph: &'a TransitGraph,
    start: &'b SnappedPoint,
    target: &'b SnappedPoint,
    start_time: u32,
) -> Itinerary<'a> {
    let result = detailed_itinerary_internal(graph, *start.index(), *target.index(), start_time);

    result
}
