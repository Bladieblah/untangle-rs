from pathlib import Path
from typing import TypeVar
from untanglers import LayoutOptimizerInt, generate_multipartite_graph

import networkx as nx
import matplotlib.pyplot as plt

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

  nx.draw_networkx(graph, nodes_to_pos(nodes), ax=axs[0], **styles)

  nodes = optimize()
  nx.draw_networkx(graph, nodes_to_pos(nodes), ax=axs[1], **styles)

  [ax.invert_yaxis() for ax in axs]

  plt.tight_layout()

  return fig



def draw_basic_example():
  nodes, edges = generate_multipartite_graph([2, 4, 3])

  def optimize():
    opt = LayoutOptimizerInt(nodes, edges)
    opt.optimize(1, 1e-3, 2, 20, 2)
    return opt.get_nodes()
  
  fig = draw_example(nodes, edges, optimize)

  fig.savefig(str(images / "basic.png"), dpi=600)

def draw_complex_example():
  nodes, edges = generate_multipartite_graph([20, 40, 30, 35, 10])

  def optimize():
    opt = LayoutOptimizerInt(nodes, edges)
    opt.optimize(10, 0.1, 5, 2000, 10)
    return opt.get_nodes()
  
  styles = {
    "node_size": 50,
    "font_size": 3,
    # "with_labels": False,
    "linewidths": 1,
  }
  fig = draw_example(nodes, edges, optimize, styles)

  fig.savefig(str(images / "complex.png"), dpi=600)


if __name__ == "__main__":
  draw_basic_example()
  draw_complex_example()
