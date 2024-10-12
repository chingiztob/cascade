import cascade
import pandas as pd

gtfs_path = "/home/chingiz/Rust/py_rust/cascade/cascade-bin/files/Saint_Petersburg"
pbf_path = "/home/chingiz/Rust/osm/roads_SZ.pbf"
departure = 20000
duration = 70000
weekday = "monday"

graph = cascade.create_graph(gtfs_path, pbf_path, departure, duration, weekday)

point_1 = (30.349061, 59.878163)
point_2 = (30.370268, 59.851074)
list_of_points = [point_1, point_2]

result = cascade.calculate_od_matrix(graph, list_of_points, 43200)

# crate dataframe from result dict of dict
df: pd.DataFrame = pd.DataFrame.from_dict(result, orient="index").fillna(0)
# Convert the dict of dicts into a DataFrame

# Convert to long format using stack
df_long = df.stack().reset_index()
df_long.columns = ['from_point', 'to_point', 'weight']
df.to_csv("result.csv")

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
