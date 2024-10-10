use std::collections::HashMap;

use geo::Point;

use crate::prelude::*;

#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn create_graph(
    gtfs_path: &str,
    pbf_path: &str,
    departure: u32,
    duration: u32,
    weekday: &str,
) -> TransitGraph {
    let feed_args = FeedArgs {
        gtfs_path,
        pbf_path,
        departure,
        duration,
        weekday,
    };
    let instant = std::time::Instant::now();
    let transit_graph = TransitGraph::from(&feed_args).unwrap();
    println!("Graph creation time: {:?}", instant.elapsed());

    transit_graph
}

#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn shortest_path_wrapper(
    graph: &TransitGraph,
    start_time: u32,
    x: f64,
    y: f64,
) -> HashMap<usize, f64> {
    let source = SnappedPoint::init(Point::new(x, y), graph).unwrap();

    let hmap = single_source_shortest_path(graph, &source, start_time);

    hmap.into_iter().map(|(k, v)| (k.index(), v)).collect()
}

#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn single_shortest_path_wrapper(graph: &TransitGraph, start_time: u32) -> f64 {
    let source = SnappedPoint::init(Point::new(30.320234, 59.875912), graph).unwrap();
    let target = SnappedPoint::init(Point::new(30.309416, 60.066852), graph).unwrap();

    shortest_path(graph, &source, &target, start_time).unwrap()
}
