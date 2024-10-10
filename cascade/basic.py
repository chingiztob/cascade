import time

from cascade import _cascade_core as core
from cascade._cascade_core import TransitGraphRs


class TransitGraph:
    """
    Efficient implemenration of graph from Rust
    """

    def __init__(self, graph: TransitGraphRs):
        self.graph = graph

    def get_graph(self):
        return self.graph


def single_source_shortest_path(graph: TransitGraph, dep_time: int):
    start_time = time.perf_counter()

    travel_time = core.single_source_shortest_path(graph.get_graph(), dep_time)
    print("Time elapsed", time.perf_counter() - start_time)

    return travel_time


def shortest_path(graph: TransitGraph, dep_time: int):
    start_time = time.perf_counter()

    travel_time = core.shortest_path(graph.get_graph(), dep_time)
    print("Time elapsed", time.perf_counter() - start_time)

    return travel_time


def create_graph(
    gtfs_path: str, pbf_path: str, departure: int, duration: int, weekday: str
) -> TransitGraph:
    return TransitGraph(
        core.create_graph(gtfs_path, pbf_path, departure, duration, weekday)
    )
