import numpy as np
import persistence

# Example: 4 points in 3D space
points = np.array([
    [1.0, 2.0, 3.0],
    [4.0, 5.0, 6.0],
    [7.0, 8.0, 9.0],
    [2.0, 3.0, 4.0]
])

distances = persistence.pairwise_distances_py(points)
print(distances)