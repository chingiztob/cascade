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

///  Finds the shortest paths from source node in a time-dependent graph using Dijkstra's algorithm.
/// # Arguments
/// * `graph` - A reference to a `TransitGraph` object.
/// * `start` - The source node index.
/// * `start_time` - The starting time in seconds since midnight.
/// # Returns
/// A `HashMap` with the shortest path weight in seconds to each node from the source node.
/// # Implementation notes
/// This function uses a priority queue to explore the graph with
/// almost classic Dijkstra's algorithm. The main difference is that the
/// delay between two nodes is calculated based on the `current time`
/// and the sorted schedules of the edge.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn detailed_itinerary_internal(
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

pub fn detailed_itinerary(
    graph: &TransitGraph,
    start: &SnappedPoint,
    target: &SnappedPoint,
    start_time: u32,
) -> Itinerary {
    let result = detailed_itinerary_internal(graph, *start.index(), *target.index(), start_time);

    result
}
