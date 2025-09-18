from collections import defaultdict
from pathlib import Path
from re import L
from typing import TypeVar
from untanglers import HierarchyOptimizerInt, LayoutOptimizerInt, generate_multipartite_graph

import networkx as nx
import matplotlib.pyplot as plt

from numpy import indices

T = TypeVar("T")

folder = Path(__file__).parent
images = folder / "images"


def nodes_to_pos(nodes: list[list[T]]) -> dict[T, tuple[float, float]]:
  pos = {}
  max_len = max(len(ns) for ns in nodes)

  for i, ns in enumerate(nodes):
    size = len(ns)
    offset = (size - 1) / 2
    for j, n in enumerate(ns):
      pos[n] = (i, (j - offset) * (max_len / len(ns))**0.7)

  return pos


def draw_networkx(graph, pos, **kwargs):
  if "node_shape" in kwargs and isinstance(kwargs["node_shape"], list):
    shapes = defaultdict(list)
    colors = kwargs["node_color"]
    for i, shape in enumerate(kwargs["node_shape"]):
      shapes[shape].append(i)
    for shape, indices in shapes.items():
      nodes = [list(graph.nodes)[i] for i in indices]
      kwargs["node_color"] = [colors[i] for i in indices]
      kwargs["node_shape"] = shape
      draw_networkx(graph, pos, nodelist=nodes, **kwargs)
  else:
    nx.draw_networkx(graph, pos, **kwargs)


def draw_example(nodes, edges, optimize, extra_styles = None):
  graph = nx.DiGraph()
  graph.add_nodes_from([(n, {"label": str(n)}) for ns in nodes for n in ns])
  graph.add_edges_from((l, r, {"weight": w}) for es in edges for (l, r, w) in es)

  fig, axs = plt.subplots(
    1,
    2,
    figsize=(12, 6),
  )
  styles = {
    "node_size": 400,
    "node_color": "w",
    "edgecolors": "k",
    "linewidths": 2,
  }

  if extra_styles is not None:
    styles.update(extra_styles)

  draw_networkx(graph, nodes_to_pos(nodes), ax=axs[0], **styles)
  nodes, before, after = optimize()
  axs[0].set_title(f"Before: {before} edge crossings")

  draw_networkx(graph, nodes_to_pos(nodes), ax=axs[1], **styles)
  axs[1].set_title(f"After: {after} edge crossings")

  [ax.invert_yaxis() for ax in axs]

  plt.tight_layout()

  return fig



def draw_basic_example():
  nodes, edges = generate_multipartite_graph([2, 4, 3])

  def optimize():
    opt = LayoutOptimizerInt(nodes, edges)
    before = opt.count_crossings()
    opt.optimize(1, 1e-3, 2, 20, 2)
    return opt.get_nodes(), before, opt.count_crossings()
  
  fig = draw_example(nodes, edges, optimize)

  fig.savefig(str(images / "basic.png"), dpi=600)

def draw_complex_example():
  nodes, edges = generate_multipartite_graph([20, 40, 30, 35, 10])

  def optimize():
    opt = LayoutOptimizerInt(nodes, edges)
    before = opt.count_crossings()
    opt.optimize(10, 0.1, 5, 2000, 10)
    return opt.get_nodes(), before, opt.count_crossings()
  
  styles = {
    "node_size": 50,
    "font_size": 3,
    "linewidths": 1,
  }
  fig = draw_example(nodes, edges, optimize, styles)

  fig.savefig(str(images / "complex.png"), dpi=600)


def draw_hierarchy_example():
  nodes, edges = generate_multipartite_graph([20, 40, 30, 35, 10])
  hierarchy = [
    [[4,5,6,5], [9, 11]],
    [[7, 20, 13], [27, 13]],
    [[8, 9, 6, 7], [17, 13]],
    [[8, 5, 9, 6, 7], [13, 15, 7]],
    [[5,5], [10]],
  ]

  colors = []
  for layer in hierarchy:
    for i, group in enumerate(layer[0]):
      colors += [f"C{i}"] * group

  shapes = []
  for layer in hierarchy:
    for i, group in enumerate(layer[1]):
      shapes += ["o^s"[i]] * group

  def optimize():
    opt = HierarchyOptimizerInt(nodes, edges, hierarchy)
    before = opt.count_crossings()
    opt.optimize(100, 0.1, 5, 1000, 20)
    return opt.get_nodes(), before, opt.count_crossings()
  
  styles = {
    "node_size": 50,
    "font_size": 3,
    "linewidths": 1,
    "node_color": colors,
    "node_shape": shapes,
    "edgecolors": None
  }
  fig = draw_example(nodes, edges, optimize, styles)

  fig.savefig(str(images / "hierarchy.png"), dpi=600)


if __name__ == "__main__":
  draw_basic_example()
  draw_complex_example()

  draw_hierarchy_example()
