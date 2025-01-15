import time as tme
from datetime import datetime, timedelta, timezone, time
import pandas as pd
from geopy import distance
import traveling_rustling


def fetch_distance_matrix(geocoded_data):
    origins = geocoded_data
    destinations = geocoded_data
    matrix = [
        [0 for _ in range(len(origins))] for _ in range(len(destinations))
    ]
    for i, origin in enumerate(origins):
        for j, destination in enumerate(destinations):
            if origin != (None, None) and destination != (None, None):
                matrix[i][j] = int(
                    distance.distance(origin, destination).km / 80 * 3600
                )  # 80 km/h, counting the seconds
            else:
                matrix[i][j] = float("inf")
    return matrix


data = pd.read_csv("python/example.csv")

a = 0
location_list = []
for i, row in data.iterrows():
    location_list.append(
        {
            "name": row["Name"],
            "address": row["Address"],
            "geocode": (row["Lat"], row["Lon"]),
            "working_time": row["Duration (Minutes)"],
            "time_windows": [],
        }
    )
    # here erstmal nur die Reihe aller Daten. raw.
    for idx in data.columns[5:]:
        if row[idx]:
            location_list[i]["time_windows"].append(idx)

time_windows = [[] for _ in range(len(location_list))]
for i, location in enumerate(location_list):
    for date_str in location["time_windows"]:
        date = datetime.strptime(date_str, "%d.%m.%Y").date()
        open_time = time(9, 0, 0, tzinfo=timezone.utc)
        date_utc = datetime(
            date.year,
            date.month,
            date.day,
            open_time.hour,
            open_time.minute,
            tzinfo=timezone.utc,
        )
        start = date_utc.timestamp()
        close_time = time(18, 0, 0, tzinfo=timezone.utc)
        end = (
            date_utc.replace(hour=close_time.hour, minute=close_time.minute)
        ).timestamp()
        if len(time_windows[i]) == 0:
            time_windows[i].append([int(start), int(end)])
        elif time_windows[i][-1][1] == start:
            time_windows[i][-1][1] = int(end)
        else:
            time_windows[i].append([int(start), int(end)])
for i in range(len(time_windows)):
    for j in range(len(time_windows[i])):
        time_windows[i][j] = tuple(time_windows[i][j])


working_times = [
    int(location["working_time"] * 60) for location in location_list
]

distance_matrix = fetch_distance_matrix(
    [location["geocode"] for location in location_list]
)
operation_times = (8 * 3600, 20 * 3600)
tic = tme.time()
solution = traveling_rustling.solve(
    distance_matrix,
    distance_matrix,
    working_times,
    time_windows,
    operation_times,
    1,
)
toc = tme.time()
print(f"Time to solve: {toc - tic:.2f} seconds")

lateness = solution.lateness
makespan = solution.duration
waiting_time = solution.waiting_time
travel_time = solution.traveling_time
print(solution.schedule)

print("Optimized Route:")
for i, event in enumerate(solution.schedule):
    window = event[0].window
    start = datetime.fromtimestamp(window[0], tz=timezone.utc)
    end = datetime.fromtimestamp(window[1], tz=timezone.utc)
    name = type(event[0]).__name__
    if hasattr(event[0], "location"):
        location = event[0].location
        print(
            f"{location_list[location]['name']} {datetime.strftime(start, '%d.%m.%Y %H:%M:%S')} to {datetime.strftime(end, '%d.%m.%Y %H:%M:%S')}"
        )
    # else:
    #     print(
    #         f"{name} {datetime.strftime(start, '%d.%m.%Y %H:%M:%S')} to {datetime.strftime(end, '%d.%m.%Y %H:%M:%S')}"
    #     )
print(f"Lateness: {timedelta(seconds=lateness)}")
print(f"Waiting Time: {timedelta(seconds=waiting_time)}")
print(f"Makespan (Total Operation Time): {timedelta(seconds=makespan)}")
print(f"Total Travel Time: {timedelta(seconds=travel_time)}")
print(f"Total iterations: {solution.iterations}")
print(
    f"Total time taken to solve: {solution.time_taken_microseconds / 1_000_000:.2f} seconds"
)
