# ruff: noqa: F401
from typing import Dict, List

class PyTransitGraph:
    """Multimodal graph of transit system, implemented with `PetGraph`."""

    ...

    def get_mapping(self) -> Dict[int, PyGraphNode]:
        """Get mapping of graph raw node ids to `PyGraphNode` objects."""
        ...

    def extend_with_transit(
        self, gtfs_path: str, departure: int, duration: int, weekday: str
    ) -> None:
        """
        Extends the graph with transit data from GTFS feed.

        Parameters
        ----------
        gtfs_path : str
            Path to the GTFS files.
        departure : int
            Departure time in seconds.
        duration : int
            Time period from departure for which the graph will be loaded.
        weekday : str
            Day of the week in lowercase (e.g., 'monday').
        """
        ...

class PyGraphNode:
    """Node of transit graph. Contains information about node type, id and geometry."""

    def get_node_type(self) -> str: ...

class PyPoint:
    """Spatial point with ID and x, y coords.
    Required to correctly pass data across Rust/Python ffi boundary"""

    def __new__(cls, x: float, y: float, id: str) -> PyPoint: ...

def create_graph(
    gtfs_path: str, pbf_path: str, departure: int, duration: int, weekday: str
) -> PyTransitGraph:
    """
    Creates a `PyTransitGraph` based on GTFS and OpenStreetMap data.

    Parameters
    ----------
    gtfs_path : str
        Path to the GTFS files.
    pbf_path : str
        Path to the OSM dump in .pbf format.
    departure : int
        Departure time in seconds.
    duration : int
        Time period from departure for which the graph will be loaded.
    weekday : str
        Day of the week in lowercase (e.g., 'monday').

    Returns
    -------
    graph : PyTransitGraph
        Combined multimodal graph representing transit network.
    """
    ...

def single_source_shortest_path_weight(
    graph: PyTransitGraph, dep_time: int, x: float, y: float
) -> Dict[int, float]:
    """
    Finds the shortest path from source point
    to all other nodes in a time-dependent graph using Dijkstra's algorithm.

    Parameters
    ----------
    graph : PyTransitGraph
        The graph to search for the shortest path.
    dep_time : int
        The starting time.
    x: float
        lat of source.
    y: float
        lon of source point.

    Returns
    -------
    Dic[int, float]

    Implementation
    --------------
    This function uses a priority queue to explore the graph with
    almost classic Dijkstra's algorithm. The main difference is that the
    delay between two nodes is calculated based on the ``current time``
    and the sorted schedules of the edge.

    References
    ----------
    .. [1] Gerth Stølting Brodal, Riko Jacob:
       Time-dependent Networks as Models to Achieve Fast Exact Time-table Queries.
       Electronic Notes in Theoretical Computer Science, 92:3-15, 2004.
       https://doi.org/10.1016/j.entcs.2003.12.019 [1]_
    .. [2] Bradfield:
       Shortest Path with Dijkstra's Algorithm
       Practical Algorithms and Data Structures
       https://bradfieldcs.com/algos/graphs/dijkstras-algorithm/ [2]_
    """
    ...

def shortest_path_weight(
    graph: PyTransitGraph,
    dep_time: int,
    source_x: float,
    source_y: float,
    target_x: float,
    target_y: float,
) -> float:
    """Finds the shortest path weight (seconds) between two points
    in a time-dependent graph using Dijkstra's algorithm."""
    ...

def shortest_path(
    graph: PyTransitGraph,
    dep_time: int,
    source_x: float,
    source_y: float,
    target_x: float,
    target_y: float,
) -> List[int]:
    """Finds the shortest path as NodeIndex sequence between two points
    in a time-dependent graph using Dijkstra's algorithm."""
    ...

def calculate_od_matrix(
    graph: PyTransitGraph, points: List[PyPoint], dep_time: int
) -> Dict[int, Dict[int, float]]:
    """
    Calculates the Origin-Destination (OD) matrix
    for a given list of nodes, and departure time."""
    ...

# Python implemented functions
# Path: cascade/validators.py

def validate_feed(gtfs_path: str) -> bool:
    """
    Validates the GTFS feed located at the specified path.

    This function checks for the presence of required GTFS files and validates
    their contents. It ensures that necessary columns are present and that
    relationships between IDs in different files are consistent. Additionally,
    it verifies the format of time columns in the stop_times.txt file.
    """
    ...

def detailed_itinerary(
    graph: PyTransitGraph,
    dep_time: int,
    source_x: float,
    source_y: float,
    target_x: float,
    target_y: float,
) -> str: ...
