import random
import math
import time

import matplotlib.pyplot as plt
import traveling_rustling


def generate_points(size):
    points = [
        (random.uniform(0, 100), random.uniform(0, 100))
        for _ in range(2 * size)
    ]
    return points


def split_points(points, balance=0.5):
    new_points = points.copy()
    first_size = int(len(new_points) * balance)
    return new_points[:first_size], new_points[first_size:]


def calculate_distance_matrix(points):
    size = len(points)
    distance_matrix = [[0] * size for _ in range(size)]
    for i in range(size):
        for j in range(size):
            if i != j:
                distance_matrix[i][j] = int(
                    math.sqrt(
                        (points[i][0] - points[j][0]) ** 2
                        + (points[i][1] - points[j][1]) ** 2
                    )
                )
    return distance_matrix


def plot_points(points, ax, title="Generated Points"):
    x, y = zip(*points)
    ax.scatter(x, y)
    ax.set_title(title)
    ax.set_xlabel("X")
    ax.set_ylabel("Y")


def plot_route(points, route, ax, title="Route"):
    x = [points[i][0] for i in route]
    y = [points[i][1] for i in route]
    ax.plot(x, y, "bo-")
    ax.set_title(title)
    ax.set_xlabel("X")
    ax.set_ylabel("Y")


def main(size):
    # statistics
    # total experiment time
    # what was the average speed of solving everything
    # what was the average speed of solving with init route
    # what was the average speed of solving with init route full
    statistics = {
        "experiment_time": 20,
        "number_of_experiments": 0,
        "total_time_cold_start": 0,
        "total_time_init_route": 0,
        "total_time_init_route_full": 0,
        "total_distance": 0,
        "total_distance_init_route": 0,
        "total_distance_init_route_full": 0,
    }
    total_start = time.time()
    while time.time() - total_start < statistics["experiment_time"]:
        points = generate_points(size)
        points1, points2 = split_points(points, balance=0.9)
        distance_matrix = calculate_distance_matrix(points)
        dm1, dm2 = (
            calculate_distance_matrix(points1),
            calculate_distance_matrix(points2),
        )
        solution = traveling_rustling.solve(distance_matrix)
        sol1 = traveling_rustling.solve(dm1)
        sol2 = traveling_rustling.solve(dm2)

        init_route = sol1.route + [len(sol1.route) + i for i in sol2.route]
        sol3 = traveling_rustling.solve(distance_matrix, init_route=init_route)
        sol4 = traveling_rustling.solve(
            distance_matrix, init_route=solution.route
        )

        statistics["number_of_experiments"] += 1
        statistics["total_time_cold_start"] += (
            solution.time_taken_microseconds / 1_000_000
        )
        statistics["total_time_init_route"] += (
            sol3.time_taken_microseconds / 1_000_000
        )
        statistics["total_time_init_route_full"] += (
            sol4.time_taken_microseconds / 1_000_000
        )
        statistics["total_distance"] += solution.distance
        statistics["total_distance_init_route"] += sol3.distance
        statistics["total_distance_init_route_full"] += sol4.distance

    for key, value in statistics.items():
        print(f"{key}: {value:.2f}")

    fig, (ax) = plt.subplots(3, 3, figsize=(12, 6))
    plot_points(points, ax[0][0], "all points")
    plot_route(
        points, solution.route, ax[0][1], "solution on all points (route full)"
    )
    plot_points(points1, ax[1][0], "first half of points")
    plot_route(
        points1,
        sol1.route,
        ax[1][1],
        "solution on first half of points (route 1/2)",
    )
    plot_points(points2, ax[2][0], "second half of points")
    plot_route(
        points2,
        sol2.route,
        ax[2][1],
        "solution on second half of points (route 2/2)",
    )
    plot_route(
        points, sol4.route, ax[0][2], "route when starting with route full"
    )
    plot_route(
        points,
        init_route,
        ax[1][2],
        "route 1/2 and 2/2 concatenated (init route)",
    )
    plot_route(
        points, sol3.route, ax[2][2], "route when starting with init route"
    )
    plt.tight_layout()
    plt.show()


if __name__ == "__main__":
    SIZE = 20  # Adjust SIZE as needed
    main(SIZE)
