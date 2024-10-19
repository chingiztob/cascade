import warnings
import os

import polars as pl


def _validate_columns(df, required_columns, filename):
    if df.is_empty() or not all(col in df.columns for col in required_columns):
        print(f"{filename} is invalid or missing required columns {required_columns}.")
        return False
    return True


def _validate_id_rels(df1, col1, df2, col2, filename1, filename2):
    if not set(df1[col1].to_list()).issubset(set(df2[col2].to_list())):
        print(f"Mismatch in {col1} between {filename1} and {filename2}.")
        return False
    return True


def validate_feed(gtfs_path: str) -> bool:
    """docstring"""
    files = [
        "agency.txt",
        "stops.txt",
        "routes.txt",
        "trips.txt",
        "stop_times.txt",
        "calendar.txt",
    ]

    if not os.path.isdir(gtfs_path) or any(
        not os.path.isfile(os.path.join(gtfs_path, file)) for file in files
    ):
        warnings.warn("Invalid GTFS path or missing required files.")
        return False

    agency_df = pl.read_csv(os.path.join(gtfs_path, "agency.txt"))
    stops_df = pl.read_csv(os.path.join(gtfs_path, "stops.txt"))
    routes_df = pl.read_csv(os.path.join(gtfs_path, "routes.txt"))
    trips_df = pl.read_csv(os.path.join(gtfs_path, "trips.txt"))
    stop_times_df = pl.read_csv(os.path.join(gtfs_path, "stop_times.txt"), infer_schema_length=10000)

    critical_errors = not all(
        [
            _validate_columns(agency_df, ["agency_id"], "agency.txt"),
            _validate_columns(stops_df, ["stop_id"], "stops.txt"),
            _validate_columns(routes_df, ["route_id", "agency_id"], "routes.txt"),
            _validate_columns(trips_df, ["trip_id", "route_id"], "trips.txt"),
            _validate_columns(
                stop_times_df,
                ["trip_id", "stop_id", "departure_time", "arrival_time"],
                "stop_times",
            ),
            _validate_id_rels(
                routes_df, "agency_id", agency_df, "agency_id", "routes", "agency"
            ),
            _validate_id_rels(
                trips_df, "route_id", routes_df, "route_id", "trips", "routes"
            ),
            _validate_id_rels(
                stop_times_df, "trip_id", trips_df, "trip_id", "stop_times", "trips"
            ),
            _validate_id_rels(
                stop_times_df, "stop_id", stops_df, "stop_id", "stop_times", "stops"
            ),
        ]
    )

    for time_col in ["departure_time", "arrival_time"]:
        invalid_times = stop_times_df.filter(
            ~pl.col(time_col).str.contains(r"^(\d{2}):([0-5]\d):([0-5]\d)$")
        )
        if not invalid_times.is_empty():
            print(f"Invalid {time_col} format found in stop_times.txt.")
            print(f"Invalid times: {invalid_times[time_col].to_list()}")

    if critical_errors:
        print("GTFS feed contains critical errors.")
        return False
    print("GTFS feed is valid.")
    return True
