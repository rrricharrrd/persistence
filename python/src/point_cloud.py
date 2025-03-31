import numpy as np
import persistence_rs


class PointCloud:
    def __init__(self, points: np.ndarray):
        self.points = points
    
    def distances(self) -> np.ndarray:
        return persistence_rs.pairwise_distances(self.points)

    def persistence_intervals(self, max_dim: int, max_dist: float) -> tuple:
        return persistence_rs.persistence_intervals(self.points, max_dim, max_dim)