# ruff: noqa: F401
from typing import Dict


class PyTransitGraph:
    """
    Wrapper class for the TransitGraphRs struct. Implemented in Rust with `petgraph` library.
    """

    ...


def create_graph(
    gtfs_path: str, pbf_path: str, departure: int, duration: int, weekday: str
) -> PyTransitGraph: ...


def single_source_shortest_path(
    graph: PyTransitGraph, dep_time: int, x: float, y: float
) -> Dict[int, float]: ...


def shortest_path(graph: PyTransitGraph, dep_time: int) -> float: ...