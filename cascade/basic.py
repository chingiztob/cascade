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

    def demo(self):
        return demo_wrapper(self.graph)


def demo_wrapper(graph: TransitGraph):
    start_time = time.perf_counter()

    travel_time = core.demo(graph.get_graph())
    print("Time elapsed", time.perf_counter() - start_time)

    return travel_time


def create_graph(
    gtfs_path: str, pbf_path: str, departure: int, duration: int, weekday: str
) -> TransitGraph:
    return TransitGraph(core.create_graph(gtfs_path, pbf_path, departure, duration, weekday))
