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


def test_mapper():
    # Given
    points = np.array(
        [
            [0.0, 0.0],
            [0.5, 0.0],
            [1.0, 0.0],
            [1.5, 0.0],
            [1.5, 0.5],
            [1.5, 1.0],
            [1.5, 1.5],
            [1.0, 1.5],
            [0.5, 1.5],
            [0.0, 1.5],
            [0.0, 1.0],
            [0.0, 0.5],
            [2.0, 0.0],
            [2.5, 0.0],
            [3.0, 0.0],
        ]
    )

    # When
    result = persistence.mapper(points, 3, 0.51, 1)
    print(result)

    # Then
    assert len(result) == 7
