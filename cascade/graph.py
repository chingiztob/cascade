from cascade import _cascade_core as core
from cascade._cascade_core import TransitGraphRs


class TransitGraph:

    def __init__(self, graph: TransitGraphRs):
        self.graph = graph

    def get_graph(self):
        return self.graph
    
def create_graph(
    gtfs_path: str, pbf_path: str, departure: int, duration: int, weekday: str
) -> TransitGraph:
    return TransitGraph(
        core.create_graph(gtfs_path, pbf_path, departure, duration, weekday)
    )
