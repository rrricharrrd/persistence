import numpy as np
import persistence
from scipy.spatial.distance import pdist, squareform


def test_distances():
    points = np.array([
        [1.0, 2.0, 3.0],
        [4.0, 5.0, 6.0],
        [7.0, 8.0, 9.0],
        [2.0, 3.0, 4.0]
    ])

    distances = persistence.pairwise_distances(points)
    distances_scipy = squareform(pdist(points))
    print(distances)
    np.testing.assert_array_almost_equal(distances, distances_scipy)


# TODO actually test stuff
def test_square_persistence():
    square = np.array([
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ])
    distances = persistence.pairwise_distances(square)
    print(distances)
    intervals = persistence.persistence_intervals(square, 2, 10.0)
    print(intervals)