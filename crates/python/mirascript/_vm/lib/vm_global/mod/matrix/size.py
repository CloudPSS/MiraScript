from __future__ import annotations

from .....types import VmAny
from ...._helpers import _required
from ._helper import _size

__all__ = ["size"]


def size(matrix: VmAny) -> list[float]:
    matrix = _required("matrix", matrix, [])
    return [float(n) for n in _size(matrix)]
