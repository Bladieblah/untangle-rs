from math import inf
import time
from functools import wraps
from typing import Dict, List, Tuple

from loguru import logger

def timeit(func):
    @wraps(func)
    def wrapper(*args, **kwargs):
        start = time.perf_counter()
        result = func(*args, **kwargs)
        end = time.perf_counter()
        elapsed_us = (end - start) * 1_000_000  # microseconds
        logger.warning(f"{func.__name__} took {elapsed_us:.2f} µs")
        return result
    return wrapper
