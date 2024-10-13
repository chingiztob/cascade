use router_core::prelude::*;

use geo::Point;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let gtfs_path = get_user_input().unwrap();
    let gtfs_path = "files/Saint_Petersburg";
    let edgelist_path = "/home/chingiz/Rust/osm/roads_SZ.pbf";

    let feed_args = FeedArgs {
        gtfs_path,
        edgelist_path,
        departure: 0,
        duration: 90000,
        weekday: "monday",
    };

    let instant = std::time::Instant::now();
    let transit_graph = TransitGraph::from(&feed_args)?;
    println!("Graph creation time: {:?}", instant.elapsed());

    let source = SnappedPoint::init(Point::new(30.320234, 59.875912), &transit_graph)?;
    let target = SnappedPoint::init(Point::new(30.309416, 60.066852), &transit_graph)?;

    let instant = std::time::Instant::now();
    let path = single_source_shortest_path(&transit_graph, &source, 43200);

    println!("Dijkstra time: {:?}", instant.elapsed());
    println!("Path: {:?}", path.get(target.index()));

    Ok(())
}
