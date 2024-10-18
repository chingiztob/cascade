use std::path::{Path, PathBuf};

use ahash::{HashMap, HashMapExt};
use geo::Point;
use itertools::Itertools;
use petgraph::graph::Graph;
use petgraph::prelude::NodeIndex;
use petgraph::visit::EdgeRef;
use polars::chunked_array::ops::SortMultipleOptions;
use polars::io::csv::read::CsvReadOptions;
use polars::prelude::*;

use crate::graph::{GraphEdge, GraphNode, TransitEdge, TransitGraph, TransitNode, Trip};
use crate::Error;

fn read_csv(file_path: PathBuf) -> Result<DataFrame, Error> {
    let df = CsvReadOptions::default()
        .with_has_header(true)
        // This will be slow, but protects from wrong schema inference
        // Probably should be set to None (read all lines to infer schema)
        // but this dramatically increases load times (still fast though)
        .with_infer_schema_length(Some(10000))
        .try_into_reader_with_file_path(Some(file_path))?
        .finish()?;

    Ok(df)
}

fn hhmmss_to_sec(str_val: &Series) -> Series {
    str_val
        .str()
        .unwrap_or_else(|_| {
            panic!(
                "invalid time format for {}. Expected HH:MM:SS",
                str_val.name()
            )
        })
        .iter()
        .map(|opt_time: Option<&str>| {
            let time = opt_time.unwrap();
            let parts: Vec<&str> = time.split(':').collect();
            let hours = parts[0].parse::<u32>().unwrap();
            let minutes = parts[1].parse::<u32>().unwrap();
            let seconds = parts[2].parse::<u32>().unwrap();
            Some(hours * 3600 + minutes * 60 + seconds)
        })
        .collect::<UInt32Chunked>()
        .into_series()
}

fn filter_by_time(df: &mut DataFrame, departure: u32, duration: u32) -> Result<DataFrame, Error> {
    // Convert time columns to seconds since midnight
    df.apply("arrival_time", hhmmss_to_sec)?;
    df.apply("departure_time", hhmmss_to_sec)?;

    let mask = df.column("departure_time")?.gt(departure)?
        & df.column("departure_time")?.lt(departure + duration)?;

    Ok(df.filter(&mask)?)
}

pub(crate) fn prepare_dataframes<P: AsRef<Path>>(
    path: P,
    departure: u32,
    duration: u32,
    weekday: &str,
) -> Result<(DataFrame, DataFrame), Error> {
    let gtfs_path = PathBuf::from(path.as_ref());

    let mut stops_df = read_csv(gtfs_path.join("stops.txt"))?;
    let stop_times_df = read_csv(gtfs_path.join("stop_times.txt"))?;
    let mut trips_df = read_csv(gtfs_path.join("trips.txt"))?;
    let calendar_df = read_csv(gtfs_path.join("calendar.txt"))?;

    let stops_df = stops_df
        .with_column(stops_df.column("stop_id")?.cast(&DataType::String)?)?
        .clone();

    // Filter calendar for active services on specific days (e.g., day_of_week == 1)
    let service_ids = calendar_df
        .filter(&calendar_df.column(weekday)?.equal(1)?)?
        .select(["service_id"])?;

    // Join trips with active services to filter by service_id
    trips_df = trips_df.join(
        &service_ids,
        ["service_id"],
        ["service_id"],
        JoinArgs::new(JoinType::Inner),
    )?;

    // Filter stop_times by the valid trip_ids from the filtered trips_df
    let mut valid_stop_times = stop_times_df.join(
        &trips_df,
        ["trip_id"],
        ["trip_id"],
        JoinArgs::new(JoinType::Inner),
    )?;

    let filtered_stop_times_df =
        filter_by_time(&mut valid_stop_times, departure, departure + duration)?;

    println!("Filtering left {} rows", filtered_stop_times_df.height());
    Ok((stops_df, filtered_stop_times_df))
}

pub(crate) fn new_graph(
    stops_df: &DataFrame,
    stop_times_df: &DataFrame,
) -> Result<TransitGraph, Error> {
    let mut transit_graph = TransitGraph::new();
    let mut node_id_map: HashMap<String, NodeIndex> = HashMap::new();

    add_nodes_to_graph(stops_df, &mut transit_graph, &mut node_id_map)?;
    add_edges_to_graph(stop_times_df, &mut transit_graph, &node_id_map)?;
    transit_graph.sort_trips();

    Ok(transit_graph)
}

/// Add nodes to the graph from the stops `DataFrame`
/// and store the node indices in a `HashMap`
/// for later use when adding edges
/// `node_id_map` is used to store the node indices.
/// That is required to access the nodes when adding edges
/// as [`petgraph`] requires the internal node indices but
/// the `stop_id` is used as a reference in the GTFS feed.
/// # Arguments
/// * `stops_df` - A `DataFrame` containing stop information
/// * `transit_graph` - A mutable reference to the `TransitGraph`
/// * `node_id_map` - A mutable reference to a `HashMap` to store node indices
fn add_nodes_to_graph(
    stops_df: &DataFrame,
    transit_graph: &mut TransitGraph,
    node_id_map: &mut HashMap<String, NodeIndex>,
) -> Result<(), Error> {
    let stop_ids = stops_df.column("stop_id")?.str()?.iter();
    let stop_lons = stops_df.column("stop_lon")?.f64()?.iter();
    let stop_lats = stops_df.column("stop_lat")?.f64()?.iter();

    for (stop_id, (stop_lon, stop_lat)) in stop_ids.zip(stop_lons.zip(stop_lats)) {
        let stop_id = stop_id.ok_or_else(|| Error::MissingValue("stop_id".to_string()))?;
        let stop_lon = stop_lon.ok_or_else(|| Error::MissingValue("stop_lon".to_string()))?;
        let stop_lat = stop_lat.ok_or_else(|| Error::MissingValue("stop_lat".to_string()))?;

        let key = transit_graph.add_node(GraphNode::Transit(TransitNode {
            stop_id: stop_id.to_string(),
            geometry: Point::new(stop_lon, stop_lat),
        }));

        node_id_map.insert(stop_id.to_string(), key);
    }
    Ok(())
}

fn select_columns(sorted_group: &DataFrame) -> Vec<&str> {
    // Check if the wheelchair_accessible column exists and select columns accordingly
    let columns: Vec<&str> = if sorted_group
        .get_column_names()
        .contains(&&PlSmallStr::from_str("wheelchair_accessible"))
    {
        vec![
            "arrival_time",
            "departure_time",
            "stop_id",
            "stop_sequence",
            "route_id",
            "wheelchair_accessible",
        ]
    } else {
        vec![
            "arrival_time",
            "departure_time",
            "stop_id",
            "stop_sequence",
            "route_id",
        ]
    };

    columns
}

fn add_edges_to_graph(
    stop_times_df: &DataFrame,
    transit_graph: &mut TransitGraph,
    node_id_map: &HashMap<String, NodeIndex>,
) -> Result<(), Error> {
    let grouped_df = stop_times_df.group_by(["trip_id"])?;

    grouped_df.apply(|group| {
        let sorted_group = group.sort(["stop_sequence"], SortMultipleOptions::default())?;

        let columns = select_columns(&sorted_group);
        let selected_columns = sorted_group.select(columns)?;

        let stops = selected_columns
            .column("stop_id")?
            .cast(&DataType::String)?;
        let stops = stops.str()?.iter().map(|opt| opt.expect("Missing stop_id"));

        let arrival_times = selected_columns
            .column("arrival_time")?
            .cast(&DataType::UInt32)?;
        let arrival_times = arrival_times
            .u32()?
            .iter()
            .map(|opt| opt.expect("Missing arrival_time"));

        let departure_times = selected_columns
            .column("departure_time")?
            .cast(&DataType::UInt32)?;
        let departure_times = departure_times
            .u32()?
            .iter()
            .map(|opt| opt.expect("Missing departure_time"));

        let route_ids = selected_columns
            .column("route_id")?
            .cast(&DataType::String)?;
        let route_ids = route_ids
            .str()?
            .iter()
            .map(|opt| opt.expect("Missing route_id"));

        // zip all columns into a single iterator
        let zipped = stops
            .zip(arrival_times)
            .zip(departure_times)
            .zip(route_ids)
            .tuple_windows();

        for (
            (((current_stop, current_arrival_time), _), current_route_id),
            (((next_stop, _), next_arrival_time), _),
        ) in zipped
        {
            // Invalid datasets with negative edge weights
            // will cause invalid Dijkstra routing
            if current_arrival_time > next_arrival_time {
                Err(Error::NegativeWeight(format!(
                    "Negative weight detected: {current_stop} -> {next_stop}"
                )))?;
            }

            add_edge_to_graph(
                transit_graph,
                node_id_map,
                current_stop,
                next_stop,
                current_arrival_time,
                next_arrival_time,
                current_route_id,
            )?;
        }

        Ok(selected_columns)
    })?;

    Ok(())
}

fn add_edge_to_graph(
    transit_graph: &mut TransitGraph,
    node_id_map: &HashMap<String, NodeIndex>,
    current_stop: &str,
    next_stop: &str,
    current_arrival_time: u32,
    next_arrival_time: u32,
    current_route_id: &str,
) -> Result<(), Error> {
    let route = Trip::new(
        current_arrival_time,
        next_arrival_time,
        String::from(current_route_id),
        false,
    );

    let source_node = *node_id_map
        .get(current_stop)
        .ok_or_else(|| Error::NodeNotFound(String::from(current_stop)))?;
    let target_node = *node_id_map
        .get(next_stop)
        .ok_or_else(|| Error::NodeNotFound(String::from(next_stop)))?;

    if let Some(edge) = transit_graph.find_edge(source_node, target_node) {
        let edge_data = transit_graph.edge_weight_mut(edge).unwrap();
        if let GraphEdge::Transit(transit_edge) = edge_data {
            transit_edge.edge_trips.push(route);
        }
    } else {
        transit_graph.add_edge(
            source_node,
            target_node,
            GraphEdge::Transit(TransitEdge {
                edge_trips: vec![route],
            }),
        );
    }

    Ok(())
}

pub(crate) fn merge_graphs<N, E>(walk_graph: &mut Graph<N, E>, transit_graph: &Graph<N, E>)
where
    N: Clone,
    E: Clone,
{
    // Here is the idea:
    // NodeIndex is used in petgraph graph to uniquely identify nodes
    // in classic `graph` indexes are always 0..n where n is the number of nodes
    // So when we add nodes from one graph, to another (iterating in index order)
    // We can calculate which index will be assigned to the node in the new graph
    let offset = walk_graph.node_count();

    for weight in transit_graph.node_weights() {
        walk_graph.add_node(weight.clone());
    }

    for edge in transit_graph.edge_references() {
        let source = NodeIndex::new(edge.source().index() + offset);
        let target = NodeIndex::new(edge.target().index() + offset);
        let weight = edge.weight().clone();
        walk_graph.add_edge(source, target, weight);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_nodes_to_graph() {
        let df = df! {
            "stop_id" => &["A", "B", "C"],
            "stop_lon" => &[10.0, 20.0, 30.0],
            "stop_lat" => &[50.0, 60.0, 70.0]
        }
        .unwrap();

        let mut graph = TransitGraph::new();
        let mut node_id_map = HashMap::new();
        let result = add_nodes_to_graph(&df, &mut graph, &mut node_id_map);

        assert!(result.is_ok());
        assert_eq!(graph.node_count(), 3);
        assert_eq!(node_id_map.len(), 3);
    }
}
