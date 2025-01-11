import traveling_rustling

import random
import matplotlib.pyplot as plt

# Generate distance matrix based on n random integer points in [0,1000] x [0,1000] euclidian space
n = 30
points = [(random.randint(0, 1000), random.randint(0, 1000)) for _ in range(n)]
distance_matrix = [[0 for _ in range(n)] for _ in range(n)]
for i in range(n):
    for j in range(n):
        distance_matrix[i][j] = int(
            100
            * (
                (points[i][0] - points[j][0]) ** 2
                + (points[i][1] - points[j][1]) ** 2
            )
            ** 0.5
        )
import time

tic = time.time()
best_route = traveling_rustling.solve(distance_matrix)
print("Time taken: ", time.time() - tic)
plt.plot(
    [points[best_route[i % n]][0] for i in range(n + 1)],
    [points[best_route[i % n]][1] for i in range(n + 1)],
    "ro-",
)
plt.show()
