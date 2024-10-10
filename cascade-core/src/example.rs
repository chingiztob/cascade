use crate::prelude::*;

use geo::Point;

pub fn create_graph() -> TransitGraph {
    let gtfs_path = "/home/chingiz/Rust/py_rust/cascade/cascade-bin/files/Saint_Petersburg";
    let edgelist_path = "/home/chingiz/Rust/osm/roads_SZ.pbf";

    let feed_args = FeedArgs {
        gtfs_path,
        edgelist_path,
        departure: 0,
        duration: 90000,
        weekday: "monday",
    };

    let instant = std::time::Instant::now();
    let transit_graph = TransitGraph::from(&feed_args).unwrap();
    println!("Graph creation time: {:?}", instant.elapsed());

    transit_graph
}

pub fn demo(graph: &TransitGraph) -> f64 {
    let source = SnappedPoint::init(Point::new(30.320234, 59.875912), &graph).unwrap();
    let target = SnappedPoint::init(Point::new(30.309416, 60.066852), &graph).unwrap();

    *single_source_shortest_path(&graph, &source, 43200)
        .get(target.index())
        .unwrap()
}
