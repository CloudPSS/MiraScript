from ..._helpers_utils import create_module
from . import _matrix

matrix = create_module(
    "matrix", dict([[f, getattr(_matrix, f)] for f in _matrix.__all__])
)


__all__ = ["matrix"]
