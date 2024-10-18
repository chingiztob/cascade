//! Time-dependent Dijkstra's algorithm for finding the shortest paths in a time-dependent graph.
//! Algorithm is based on the classic Dijkstra's algorithm
//! with the difference that the delay between two nodes is calculated
//! based on the current time and the sorted schedules of the edge.
//! Implementation is based on classic Dijkstra's algorithm implementation in the [`petgraph`] crate
//! and Time-dependent Dijkstra's algorithm implementation in the `Nxtransit` python library.

use std::cmp::Ordering;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::BinaryHeap;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;

use crate::graph::TransitGraph;
use crate::prelude::SnappedPoint;
use crate::Error;

/// `MinScored<K, T>` holds a score `f64` and a scored object `T` in
/// a pair for use with a `BinaryHeap`.
///
/// `MinScored` compares in reverse order by the score, so that we can
/// use `BinaryHeap` as a min-heap to extract the score-value pair with the
/// least score.
/// This implementation is based on the one in the `petgraph` crate.
#[derive(Copy, Clone, PartialEq)]
struct MinScored<K>(f64, K);

impl<K: Eq> Eq for MinScored<K> {}

impl<K: PartialOrd> PartialOrd for MinScored<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl<K: Ord> Ord for MinScored<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.partial_cmp(&self.0).unwrap()
    }
}

#[must_use]
pub fn single_source_shortest_path(
    graph: &TransitGraph,
    start: &SnappedPoint,
    start_time: u32,
) -> HashMap<NodeIndex, f64> {
    let source_index = start.index();
    let distance = *start.distance();
    let mut result = time_dependent_dijkstra(graph, *source_index, None, start_time);
    // add distance to all values in the result
    result.iter_mut().for_each(|(_, v)| *v += distance);
    result
}

pub fn shortest_path(
    graph: &TransitGraph,
    start: &SnappedPoint,
    target: &SnappedPoint,
    start_time: u32,
) -> Result<f64, Error> {
    let source_index = start.index();
    let target_index = target.index();

    let distance = *start.distance();
    let result = time_dependent_dijkstra(graph, *source_index, Some(*target_index), start_time);
    // add distance to all values in the result
    let time = result.get(target_index).ok_or(Error::MissingValue(format!(
        "failed to extract time for node {target_index:?}"
    )))?;

    Ok(*time + distance)
}

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
fn time_dependent_dijkstra(
    graph: &TransitGraph,
    start: NodeIndex,
    target: Option<NodeIndex>,
    start_time: u32,
) -> HashMap<NodeIndex, f64> {
    let mut visited = HashSet::new();
    let mut scores: HashMap<NodeIndex, f64> = HashMap::new();

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
