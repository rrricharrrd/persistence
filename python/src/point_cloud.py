import numpy as np

import persistence


class PointCloud:
    def __init__(self, points: np.ndarray):
        self.points = points

    def distances(self) -> np.ndarray:
        return persistence.pairwise_distances(self.points)

    def persistence_intervals(self, max_dim: int, max_dist: float) -> tuple:
        return persistence.persistence_intervals(self.points, max_dim, max_dim)
