use crate::prelude::*;

use geo::Point;

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
pub fn shortest_path(graph: &TransitGraph) -> f64 {
    let source = SnappedPoint::init(Point::new(30.320234, 59.875912), graph).unwrap();
    let target = SnappedPoint::init(Point::new(30.309416, 60.066852), graph).unwrap();

    *single_source_shortest_path(graph, &source, 43200)
        .get(target.index())
        .unwrap()
}
