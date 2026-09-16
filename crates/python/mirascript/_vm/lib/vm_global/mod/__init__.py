from ..._helpers_utils import _create_module
from . import matrix

matrix = _create_module("matrix", matrix)

__all__ = ["matrix"]
