import numpy as np
from scipy.spatial.distance import pdist, squareform

from persistence import PointCloud


def test_distances():
    # Given
    points = np.array([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0], [2.0, 3.0, 4.0]])
    cloud = PointCloud(points)

    # When
    distances = cloud.distances(points)
    print(distances)

    # Then
    distances_scipy = squareform(pdist(points))
    np.testing.assert_array_almost_equal(distances, distances_scipy)


def test_square_persistence():
    # Given
    square = np.array(
        [
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ]
    )
    cloud = PointCloud(square)

    # When
    distances = cloud.distances(square)
    print(distances)
    intervals = cloud.persistence_intervals(square, 2, 10.0)
    print(intervals)

    # Then
    # TODO actually test stuff
