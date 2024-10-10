/*!
`NxTransit` is a Rust package designed for creating a multimodal graph
representation of public transportation systems. It uses General Transit
Feed Specification (GTFS) data to construct the graph and perform various
time-dependent calculations.

# Key Features:
- **Multimodal Graph Creation:** `NxTransit` can generate a graph that
  integrates different modes of transportation with street networks.
- **Time-Dependent Calculations:** The package allows for the analysis of
  transit dynamics by considering the time-dependency of transit schedules.
  This includes calculating shortest paths with departure times, travel
  time matrices, and service frequency.
- **GTFS Data Support:** `NxTransit` uses GTFS data, a common format for public
  transportation schedules and associated geographic information, as the
  basis for graph construction and analysis.

# Example
```ignore
use polars_test::prelude::*;

use geo::Point;

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

let transit_graph = TransitGraph::from(&feed_args).unwrap();
let source = SnappedPoint::init(Point::new(30.320234, 59.875912), &transit_graph).unwrap();
let target = SnappedPoint::init(Point::new(30.309416, 60.066852), &transit_graph).unwrap();

let path = single_source_shortest_path(&transit_graph, &source, 43200);

println!("Path: {:?}", path.get(target.index()));
```
*/

use osm4routing::NodeId;
use polars::prelude::*;
use thiserror::Error;

pub mod algo;
pub mod connectors;
pub mod graph;
pub mod loaders;
pub mod prelude;
pub mod streets;
pub mod utils;
pub mod example;

const WALK_SPEED: f64 = 1.39;

/// Error type for `TransitGraph`
#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to cast column: {0}")]
    CastError(String),
    #[error("Numeric cast error: {0}")]
    CastErrorNumeric(#[from] std::num::TryFromIntError),
    #[error("Numeric parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Missing column: {0}")]
    MissingColumn(String),
    #[error("Hashmap does not contain key: {0:?}")]
    MissingKey(NodeId),
    #[error("Missing value in column: {0}")]
    MissingValue(String),
    #[error("Negative weight detected: {0}")]
    NegativeWeight(String),
    #[error("Node not found for id: {0}")]
    NodeNotFound(String),
    #[error("Polars error: {0}")]
    PolarsError(#[from] PolarsError),
}

impl From<Error> for PolarsError {
    fn from(err: Error) -> Self {
        match err {
            Error::PolarsError(e) => e,
            _ => Self::ComputeError(err.to_string().into()),
        }
    }
}
