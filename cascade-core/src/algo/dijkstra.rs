//! Time-dependent Dijkstra's algorithm for finding the shortest paths in a time-dependent graph.
//! Algorithm is based on the classic Dijkstra's algorithm
//! with the difference that the delay between two nodes is calculated
//! based on the current time and the sorted schedules of the edge.
//! Implementation is based on classic Dijkstra's algorithm implementation in the [`petgraph`] crate
//! and Time-dependent Dijkstra's algorithm implementation in the `Nxtransit` python library.

use std::collections::BinaryHeap;

use hashbrown::hash_map::Entry::{Occupied, Vacant};
use hashbrown::{HashMap, HashSet};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;

use crate::algo::MinScored;
use crate::graph::TransitGraph;

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
pub(crate) fn time_dependent_dijkstra(
    graph: &TransitGraph,
    start: NodeIndex,
    target: Option<NodeIndex>,
    start_time: u32,
) -> HashMap<NodeIndex, f64> {
    let mut visited = HashSet::new();
    let mut scores: HashMap<NodeIndex, f64> =
        HashMap::with_capacity(graph.into_inner_graph().node_count());

    let mut visit_next = BinaryHeap::new();
    scores.insert(start, 0.0);
    visit_next.push(MinScored(0.0, (start, start_time)));

    while let Some(MinScored(node_score, (node, current_time))) = visit_next.pop() {
        if visited.contains(&node) {
            continue;
        }

        if let Some(target) = target {
            if node == target {
                break;
            }
        }

        for edge in graph.edges(node) {
            let next = edge.target();
            if visited.contains(&next) {
                continue;
            }

            let travel_time = edge.weight().calculate_delay(current_time);
            if travel_time.is_infinite() {
                continue;
            }

            let next_score = node_score + travel_time;
            let next_time = current_time + travel_time as u32;

            match scores.entry(next) {
                Occupied(mut ent) => {
                    if next_score < *ent.get() {
                        ent.insert(next_score);
                        visit_next.push(MinScored(next_score, (next, next_time)));
                    }
                }
                Vacant(ent) => {
                    ent.insert(next_score);
                    visit_next.push(MinScored(next_score, (next, next_time)));
                }
            }
        }
        visited.insert(node);
    }
    scores
}
