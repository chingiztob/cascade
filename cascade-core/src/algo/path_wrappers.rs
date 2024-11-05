//! Time-dependent Dijkstra's algorithm for finding the shortest paths in a time-dependent graph.
//! Algorithm is based on the classic Dijkstra's algorithm
//! with the difference that the delay between two nodes is calculated
//! based on the current time and the sorted schedules of the edge.
//! Implementation is based on classic Dijkstra's algorithm implementation in the [`petgraph`] crate
//! and Time-dependent Dijkstra's algorithm implementation in the `Nxtransit` python library.

use ahash::HashMap;
use petgraph::graph::NodeIndex;

use crate::algo::dijkstra::{time_dependent_dijkstra, time_dependent_dijkstra_path};
use crate::graph::TransitGraph;
use crate::prelude::SnappedPoint;
use crate::Error;

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

pub fn shortest_path_weight(
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

pub fn shortest_path(
    graph: &TransitGraph,
    start: &SnappedPoint,
    target: &SnappedPoint,
    start_time: u32,
) -> Vec<usize> {
    time_dependent_dijkstra_path(graph, *start.index(), Some(*target.index()), start_time)
        .into_iter()
        .map(petgraph::prelude::NodeIndex::index)
        .collect()
}
