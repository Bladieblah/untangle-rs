# Original python implementation this module is based on

import itertools
import random
from collections import defaultdict
from copy import deepcopy
from typing import Dict, Generic, List, Tuple, TypeVar

T = TypeVar("T")


class Crossings(Generic[T]):
    def __init__(
        self,
        left: List[T],
        right: List[T],
        edges: List[Tuple[T, T, int]],
    ):
        self.left = deepcopy(left)
        self.right = deepcopy(right)
        self.edges = deepcopy(edges)

        self.size_left = len(self.left)
        self.size_right = len(self.right)

    def count_crossings(self):
        index_left = {n: i for i, n in enumerate(self.left)}
        index_right = {n: i for i, n in enumerate(self.right)}

        try:
            edges = sorted((index_left[l], index_right[r], w) for l, r, w in self.edges)
        except:
            # print(self.edges)
            raise
        weights = [0] * len(self.right)
        crossings = 0

        for _, right, weight in edges:
            crossings += weight * sum(weights[right + 1 :])
            weights[right] += weight

        return crossings

    def initialize_node_order(self):
        left_order, right_order = [], []
        for left, right, _ in sorted(self.edges, key=lambda x: -x[2]):
            if left not in left_order:
                left_order.append(left)
            if right not in right_order:
                right_order.append(right)

        self.left = left_order + [n for n in self.left if n not in left_order]
        self.right = right_order + [n for n in self.right if n not in right_order]

    def count_pair_crossings(self):
        weights: Dict[T, Dict[int, int]] = defaultdict(dict)
        index_right = {n: i for i, n in enumerate(self.right)}
        pair_crossings: Dict[Tuple[T, T], Tuple[int, int]] = {}

        for left, right, weight in self.edges:
            weights[left][index_right[right]] = weight

        cumulative_weights_f: Dict[T, List[int]] = {}
        cumulative_weights_b: Dict[T, List[int]] = {}
        for k, v in weights.items():
            cumulative_weights_f[k] = [0] * self.size_right
            for i in range(1, self.size_right):
                cumulative_weights_f[k][i] = cumulative_weights_f[k][i - 1] + v.get(
                    i - 1, 0
                )

            cumulative_weights_b[k] = [0] * self.size_right
            for i in list(range(self.size_right - 1))[::-1]:
                cumulative_weights_b[k][i] = cumulative_weights_b[k][i + 1] + v.get(
                    i + 1, 0
                )

        for a, b in itertools.combinations(self.left, 2):
            crossings = [0, 0]
            if b in cumulative_weights_b:
                for j in range(self.size_right):
                    if (w_a := weights[a].get(j, 0)) != 0:
                        crossings[0] += w_a * cumulative_weights_f[b][j]
                        crossings[1] += w_a * cumulative_weights_b[b][j]

            pair_crossings[(a, b)] = (crossings[0], crossings[1])
            pair_crossings[(b, a)] = (crossings[1], crossings[0])

        return pair_crossings

    def swap_neighbours(self, max_iterations: int = 100):
        est_crossings = self.count_crossings()
        if est_crossings == 0:
            return

        pair_crossings = self.count_pair_crossings()
        for _ in range(max_iterations):
            for j in range(self.size_left - 1):
                a, b = self.left[j : j + 2]
                keep, swap = pair_crossings[(a, b)]
                if swap < keep or swap == keep and random.random() < 0.3:
                    self.left[j : j + 2] = [b, a]
                    est_crossings += swap - keep
            if est_crossings == 0:
                break
