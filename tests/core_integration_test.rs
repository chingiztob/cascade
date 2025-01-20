use cascade_core::algo::detailed_itinerary;
use cascade_core::prelude::*;
use geo::Point;
use std::path::PathBuf;

#[allow(clippy::cast_possible_truncation)]
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

    let transit_graph = TransitGraph::from(feed_args).expect("Failed to construct Transit Graph");

    let source = SnappedPoint::init(Point::new(93.528906, 56.245849), &transit_graph)
        .expect("Failed to concstruct snapped point");
    let target = SnappedPoint::init(Point::new(93.554203, 56.237849), &transit_graph)
        .expect("Failed to concstruct snapped point");

    let weights = single_source_shortest_path_weight(&transit_graph, &source, departure_time);
    let path = detailed_itinerary(&transit_graph, &source, &target, departure_time, false);

    let weight = weights.get(target.index()).expect("Node should be reached");

    assert_eq!(*weight as i32, 1121);
    assert!(matches!(
        path.segments.last().unwrap(),
        cascade_core::algo::itinerary::segment::Segment::Pedestrian {
            weight: _,
            geometry: _
        }
    ));
}
