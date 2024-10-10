import time

from .graph import TransitGraph
from cascade import _cascade_core as core


def single_source_shortest_path(graph: TransitGraph, dep_time: int, x: float, y: float):
    start_time = time.perf_counter()

    travel_time = core.single_source_shortest_path_rs(graph.get_graph(), dep_time, x, y)
    print("Time elapsed", time.perf_counter() - start_time)

    return travel_time


def shortest_path(graph: TransitGraph, dep_time: int):
    start_time = time.perf_counter()

    travel_time = core.shortest_path_rs(graph.get_graph(), dep_time)
    print("Time elapsed", time.perf_counter() - start_time)

    return travel_time
