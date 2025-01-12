use cascade_core::algo::detailed_itinerary;
use cascade_core::prelude::*;
use geo::Point;
use std::path::PathBuf;

#[test]
fn main_zheleznogorsk_test() {
    let gtfs_path: PathBuf = "tests/test_data/Zhelez".into();
    let pbf_path: PathBuf = "tests/test_data/roads_zhelez.pbf".into();

    let feed_args = FeedArgs {
        gtfs_path,
        pbf_path,
        departure: 0,
        duration: 90000,
        weekday: "monday",
    };
    let departure_time = 43200;

    let instant = std::time::Instant::now();
    let transit_graph = TransitGraph::from(feed_args).unwrap();
    println!("Graph creation time: {:?}", instant.elapsed());

    let source = SnappedPoint::init(Point::new(93.528906, 56.245849), &transit_graph).unwrap();
    let target = SnappedPoint::init(Point::new(93.554203, 56.237849), &transit_graph).unwrap();
    let instant = std::time::Instant::now();

    let weights = single_source_shortest_path_weight(&transit_graph, &source, departure_time);
    let path = detailed_itinerary(&transit_graph, &source, &target, departure_time);

    println!("Path: {path:#?}");
    println!("Weight: {:#?}", weights.get(target.index()));
    println!("Dijkstra time: {:?}", instant.elapsed());
}
