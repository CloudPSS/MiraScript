from ..._helpers_utils import _create_module
from . import _matrix

matrix = _create_module("matrix", _matrix)

__all__ = ["matrix"]
