import cascade

gtfs_path = "/home/chingiz/Rust/py_rust/cascade/cascade-bin/files/Saint_Petersburg"
pbf_path = "/home/chingiz/Rust/osm/roads_SZ.pbf"
departure = 20000
duration = 70000
weekday = "monday"

graph = cascade.create_graph(gtfs_path, pbf_path, departure, duration, weekday)

mapping = graph.get_mapping()
print(mapping[111].get_geometry())
print(mapping[111].get_id())
print(mapping[111].get_node_type())

print(len(cascade.single_source_shortest_path(graph, 43200, 30.320234, 59.875912)))

print(
    cascade.shortest_path(
        graph=graph,
        dep_time=43200,
        source_x=30.349061,
        source_y=59.878163,
        target_x=30.370268,
        target_y=59.851074,
    )
)
