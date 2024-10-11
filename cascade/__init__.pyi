# ruff: noqa: F401
from typing import Dict, Tuple


class PyTransitGraph:
    """
    Wrapper class for the TransitGraphRs struct. Implemented in Rust with `petgraph` library.
    """

    ...
    
    def get_mapping(self) -> Dict[int, PyGraphNode]: ...

class PyGraphNode:
    """Node of transit graph."""
    
    def get_node_type(self) -> str: ...
    def get_id(self) -> str: ...
    def get_geometry(self) -> Tuple[float, float]: ...


def create_graph(
    gtfs_path: str, pbf_path: str, departure: int, duration: int, weekday: str
) -> PyTransitGraph: ...


def single_source_shortest_path(
    graph: PyTransitGraph, dep_time: int, x: float, y: float
) -> Dict[int, float]: ...

def shortest_path(
    graph: PyTransitGraph, dep_time: int, source_x: float, source_y: float, target_x: float, target_y: float
    ) -> float: ...