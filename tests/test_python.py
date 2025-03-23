import numpy as np
import persistence

# TODO actually test stuff
def test_distances():
    points = np.array([
        [1.0, 2.0, 3.0],
        [4.0, 5.0, 6.0],
        [7.0, 8.0, 9.0],
        [2.0, 3.0, 4.0]
    ])

    distances = persistence.pairwise_distances(points)
    print(distances)


def test_square_persistence():
    square = np.array([
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ])
    distances = persistence.pairwise_distances(square)
    print(distances)
    intervals = persistence.persistence_intervals(square, 1, 10.0)
    print(intervals)