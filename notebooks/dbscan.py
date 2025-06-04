# ---
# jupyter:
#   jupytext:
#     text_representation:
#       extension: .py
#       format_name: percent
#       format_version: '1.3'
#       jupytext_version: 1.17.2
#   kernelspec:
#     display_name: tda
#     language: python
#     name: tda
# ---

# %%
import matplotlib.pyplot as plt
import numpy as np
from sklearn.datasets import make_circles, make_moons

import persistence

# %%
X, y = make_circles(factor=0.3, random_state=123)
X.shape, y.shape

# %%
plt.title("Ground truth labels")
for label in np.unique(y):
    plt.scatter(*X[np.where(y == label)[0]].T)

# %%
clusters = persistence.dbscan(X, epsilon=0.2, min_points=2)
clusters

# %%
plt.title("DBSCAN cluster labels")
for label in np.unique(clusters):
    plt.scatter(*X[np.where(clusters == label)[0]].T)

# %%

# %%
X, y = make_moons()
X.shape, y.shape

# %%
plt.title("Ground truth labels")
for label in np.unique(y):
    plt.scatter(*X[np.where(y == label)[0]].T)

# %%
clusters = persistence.dbscan(X, epsilon=0.2, min_points=2)
clusters

# %%
plt.title("DBSCAN cluster labels")
for label in np.unique(clusters):
    plt.scatter(*X[np.where(clusters == label)[0]].T)

# %%
