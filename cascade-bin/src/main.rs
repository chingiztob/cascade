use cascade_core::prelude::*;
use cascade_core::algo::detailed_itinerary;

use geo::Point;
use std::path::PathBuf;
use wkt::ToWkt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let gtfs_path = get_user_input().unwrap();
    let gtfs_path: PathBuf = "files/SPB".into();
    let pbf_path: PathBuf = "/home/chingiz/Rust/osm/roads_SZ.pbf".into();

    let feed_args = FeedArgs {
        gtfs_path,
        pbf_path,
        departure: 0,
        duration: 90000,
        weekday: "monday",
    };

    let instant = std::time::Instant::now();
    let transit_graph = TransitGraph::from(feed_args)?;
    println!("Graph creation time: {:?}", instant.elapsed());

    let source = SnappedPoint::init(Point::new(30.221418, 59.851960), &transit_graph)?;
    let target = SnappedPoint::init(Point::new(30.5502047, 59.978989), &transit_graph)?;

    let instant = std::time::Instant::now();
    //let path = single_source_shortest_path_weight(&transit_graph, &source, 43200);

    let path = detailed_itinerary(&transit_graph, &source, &target, 43200);

    println!("Path: {:#?}", path);
    println!("Dijkstra time: {:?}", instant.elapsed());
    //println!("Path: {:?}", path.get(target.index()));
    println!("Path: {:#?}", path.combined_geometry().wkt_string());

    Ok(())
}
