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

  for i, ns in enumerate(nodes):
    size = len(ns)
    offset = (size - 1) / 2
    for j, n in enumerate(ns):
      pos[n] = (i, j - offset)

  return pos


def draw_basic_example():
  nodes, edges = generate_multipartite_graph([2, 4, 3])
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

  nx.draw_networkx(graph, nodes_to_pos(nodes), ax=axs[0], **styles)

  opt = LayoutOptimizerInt(nodes, edges)
  opt.optimize(1e-4, 1e-5, 1, 20, 2)
  nodes = opt.get_nodes()
  nx.draw_networkx(graph, nodes_to_pos(nodes), ax=axs[1], **styles)

  [ax.invert_yaxis() for ax in axs]

  plt.tight_layout()

  fig.savefig(str(images / "basic.png"))


if __name__ == "__main__":
  draw_basic_example()
