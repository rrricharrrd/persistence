import numpy as np

# TODO fix python packaging
import persistence


def test_dbscan():
    # Given
    points = np.array(
        [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 2.0], [0.0, 10.0], [1.0, 10.0], [0.0, 11.0], [10.0, 0.0]]
    )
    expected = np.array([1, 1, 1, 1, 2, 2, 2, 0])

    # When
    result = persistence.dbscan(points, 1.5, 2)
    print(result)

    # Then
    np.testing.assert_array_almost_equal(result, expected)
