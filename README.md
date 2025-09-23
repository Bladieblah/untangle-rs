[![PyPI version](https://img.shields.io/pypi/v/untangle-rs)](https://pypi.org/project/untangle-rs/)
[![PyPI downloads](https://img.shields.io/pypi/dm/untangle-rs)](https://pypi.org/project/untangle-rs/)
[![Coverage](https://codecov.io/gh/Bladieblah/untangle-rs/main/graph/badge.svg)](https://codecov.io/gh/Bladieblah/untangle-rs)
[![Build](https://github.com/Bladieblah/untangle-rs/actions/workflows/CI.yml/badge.svg?branch=main)](https://github.com/Bladieblah/untangle-rs/actions)

# untangle-rs
Library that minimises edge crossings in weighted multipartite graphs, with support for respecting node hierarchy. It uses a simple metropolis-hastings algorithm that iteratitely swaps neighboring nodes in order to minimize the number of crossings. It is unlikely to find a global minimum but does converge to optimal solutions.

![Basic example](docs/images/basic.png)
*Basic example*

## Basic layouts

You can minimize the crossings in arbitrary multipartite graphs, as long as you provide the parts of the graph:

```python
from untanglers import LayoutOptimizerInt

nodes = [
  [0,1,2],
  [3,4,5],
  [6,7,8],
]

edges = [
  # node_a, node_b, edge_weight
  [(0, 4, 1), (1, 3, 5)],
  [(4, 8, 2), (5, 6, 1)]
]

optimizer = LayoutOptimizerInt(nodes, edges)
edge_crossings = optimizer.count_crossings()
new_crossings = optimizer.optimize(
  start_temp = 1.0,
  end_temp = 0.1,
  steps = 3,
  max_iterations = 20,
  passes = 5,
)
```

![Complicated example](docs/images/complex.png)
*More complicated graph*

## Hierarchical layouts

In case certain nodes need to remain grouped together, the optimizer also supports hierarchy. This is useful for visualizing e.g. data lineage where columns in a table should remain together.

```python
from untanglers import HierarchyOptimizerInt

nodes = ...
edges = ...

hierarchy = [
  # 2 levels, note that the coarser level is aligned with the finer level
  [[4,5,6,5], [9, 11]],
  [[7, 20, 13], [27, 13]],
  [[8, 9, 6, 7], [17, 13]],
]

optimizer = HierarchyOptimizerInt(nodes, edges, hierarchy)
new_crossings = optimizer.optimize(...)
```

![Hierarchical example](docs/images/hierarchy.png)
*Graph with 2 levels of node hierarchy*
