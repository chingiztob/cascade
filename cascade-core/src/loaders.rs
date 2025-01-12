use std::path::{Path, PathBuf};

use ahash::{HashMap, HashMapExt};
use geo::Point;
use itertools::Itertools;
use petgraph::graph::{DiGraph, Graph};
use petgraph::prelude::NodeIndex;
use petgraph::visit::EdgeRef;
use polars::chunked_array::ops::SortMultipleOptions;
use polars::io::csv::read::CsvReadOptions;
use polars::prelude::*;

use crate::graph::{GraphEdge, GraphNode, TransitEdge, TransitNode, Trip};
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

fn hhmmss_to_sec(str_val: &Column) -> Series {
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

    let mask = df
        .column("departure_time")?
        .as_materialized_series()
        .gt(departure)?
        & df.column("departure_time")?
            .as_materialized_series()
            .lt(departure + duration)?;

    Ok(df.filter(&mask)?)
}

fn validate_feed(path: &impl AsRef<Path>) -> Result<(), Error> {
    let path = path.as_ref();

    if !path.is_dir() {
        return Err(Error::InvalidData("Invalid directory".to_string()));
    }

    let required = ["stops.txt", "trips.txt", "stop_times.txt", "calendar.txt"];

    for file in required {
        if !path.join(file).exists() {
            return Err(Error::InvalidData(format!(
                "required file {file} not exists"
            )));
        }
    }

    Ok(())
}

pub(crate) fn prepare_dataframes<P: AsRef<Path>>(
    path: P,
    departure: u32,
    duration: u32,
    weekday: &str,
) -> Result<(DataFrame, DataFrame), Error> {
    validate_feed(&path)?;
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
        .filter(
            &calendar_df
                .column(weekday)?
                .as_materialized_series()
                .equal(1)?,
        )?
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
) -> Result<DiGraph<GraphNode, GraphEdge>, Error> {
    let mut transit_graph = DiGraph::<GraphNode, GraphEdge>::new();
    let mut node_id_map: HashMap<String, NodeIndex> = HashMap::new();

    add_nodes_to_graph(stops_df, &mut transit_graph, &mut node_id_map)?;
    add_edges_to_graph(stop_times_df, &mut transit_graph, &node_id_map)?;

    // sort trips by departure time
    for edge in transit_graph.edge_weights_mut() {
        if let GraphEdge::Transit(transit_edge) = edge {
            transit_edge.edge_trips.sort();
        }
    }

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
    transit_graph: &mut DiGraph<GraphNode, GraphEdge>,
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
        .get_column_names_str()
        .contains(&"wheelchair_accessible")
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
    transit_graph: &mut DiGraph<GraphNode, GraphEdge>,
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

        // Edge weight will be defined by difference between arrival on source stop
        // and departure_time on target stop
        for (
            (((current_stop, _arrive_to_current_stop), depart_from_current_stop), current_route_id),
            (((next_stop, arrive_to_next_stop), _depart_from_next_stop), _),
        ) in zipped
        {
            // Invalid datasets with negative edge weights
            // will cause invalid Dijkstra routing
            if depart_from_current_stop > arrive_to_next_stop {
                Err(Error::NegativeWeight(format!(
                    "Negative weight detected: {current_stop} -> {next_stop},
                    {depart_from_current_stop} -> {arrive_to_next_stop}, route: {current_route_id}"
                )))?;
            }

            add_edge_to_graph(
                transit_graph,
                node_id_map,
                current_stop,
                next_stop,
                depart_from_current_stop,
                arrive_to_next_stop,
                current_route_id,
            )?;
        }

        Ok(selected_columns)
    })?;

    Ok(())
}

fn add_edge_to_graph(
    transit_graph: &mut DiGraph<GraphNode, GraphEdge>,
    node_id_map: &HashMap<String, NodeIndex>,
    current_stop: &str,
    next_stop: &str,
    depart_from_current_stop: u32,
    arrive_to_next_stop: u32,
    current_route_id: &str,
) -> Result<(), Error> {
    let route = Trip::new(
        depart_from_current_stop,
        arrive_to_next_stop,
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

        let mut graph = DiGraph::<GraphNode, GraphEdge>::new();
        let mut node_id_map = HashMap::new();
        let result = add_nodes_to_graph(&df, &mut graph, &mut node_id_map);

        assert!(result.is_ok());
        assert_eq!(graph.node_count(), 3);
        assert_eq!(node_id_map.len(), 3);
    }

    #[test]
    fn test_add_edges_to_graph() {
        let mut df = df! {
            "trip_id" => &["T1", "T1", "T1"],
            "stop_id" => &["A", "B", "C"],
            "arrival_time" => &["08:00:00", "08:10:00", "08:25:00"],
            "departure_time" => &["08:05:00", "08:15:00", "08:25:00"],
            "stop_sequence" => &[1, 2, 3],
            "route_id" => &["R1", "R1", "R1"]
        }
        .unwrap();

        let mut graph = DiGraph::<GraphNode, GraphEdge>::new();
        let mut node_id_map = HashMap::new();
        let stops_df = df! {
            "stop_id" => &["A", "B", "C"],
            "stop_lon" => &[10.0, 20.0, 30.0],
            "stop_lat" => &[50.0, 60.0, 70.0]
        }
        .unwrap();

        add_nodes_to_graph(&stops_df, &mut graph, &mut node_id_map).unwrap();

        df.apply("arrival_time", hhmmss_to_sec).unwrap();
        df.apply("departure_time", hhmmss_to_sec).unwrap();

        let result = add_edges_to_graph(&df, &mut graph, &node_id_map);

        assert!(result.is_ok());
        assert_eq!(graph.edge_count(), 2);

        let edges: Vec<_> = graph.edge_references().collect();
        assert_eq!(edges.len(), 2);

        let first_edge = edges[0];
        let second_edge = edges[1];

        assert_eq!(first_edge.source().index(), 0);
        assert_eq!(first_edge.target().index(), 1);
        assert_eq!(second_edge.source().index(), 1);
        assert_eq!(second_edge.target().index(), 2);

        if let GraphEdge::Transit(transit_edge1) = first_edge.weight() {
            assert_eq!(transit_edge1.edge_trips.len(), 1);
            assert_eq!(transit_edge1.edge_trips[0].departure_time, 29100); // 08:05:00 in seconds
            assert_eq!(transit_edge1.edge_trips[0].arrival_time, 29400); // 08:10:00 in seconds
            assert_eq!(transit_edge1.edge_trips[0].route_id, "R1");
        } else {
            panic!("Expected TransitEdge");
        }

        if let GraphEdge::Transit(transit_edge2) = second_edge.weight() {
            assert_eq!(transit_edge2.edge_trips.len(), 1);
            assert_eq!(transit_edge2.edge_trips[0].departure_time, 29700); // 08:15:00 in seconds
            assert_eq!(transit_edge2.edge_trips[0].arrival_time, 30300); // 08:25:00 in seconds
            assert_eq!(transit_edge2.edge_trips[0].route_id, "R1");
        } else {
            panic!("Expected TransitEdge");
        }
    }

    #[test]
    fn test_merge_graphs() {
        let mut walk_graph = DiGraph::<GraphNode, GraphEdge>::new();
        let mut transit_graph = DiGraph::<GraphNode, GraphEdge>::new();

        let _ = walk_graph.add_node(GraphNode::Transit(TransitNode {
            stop_id: "A".to_string(),
            geometry: Point::new(10.0, 50.0),
        }));

        let node_b = transit_graph.add_node(GraphNode::Transit(TransitNode {
            stop_id: "B".to_string(),
            geometry: Point::new(20.0, 60.0),
        }));
        let node_c = transit_graph.add_node(GraphNode::Transit(TransitNode {
            stop_id: "C".to_string(),
            geometry: Point::new(30.0, 70.0),
        }));

        transit_graph.add_edge(
            node_b,
            node_c,
            GraphEdge::Transit(TransitEdge {
                edge_trips: vec![Trip::new(0, 10, "R1".to_string(), false)],
            }),
        );

        merge_graphs(&mut walk_graph, &transit_graph);

        assert_eq!(walk_graph.node_count(), 3);
        assert_eq!(walk_graph.edge_count(), 1);
    }
}
