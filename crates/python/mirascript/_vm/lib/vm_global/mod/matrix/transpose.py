from __future__ import annotations

from .....types import VmAny, VmConst
from .....operations import Cp
from ...._helpers import _required
from ._helper import _size

__all__ = ["transpose"]


def transpose(matrix: VmAny) -> VmAny:
    matrix = _required("matrix", matrix, [])
    dims = _size(matrix)

    if len(dims) < 2:
        return matrix
    num_rows, num_cols = dims

    m: list[list[VmConst]] = matrix  # type: ignore
    transposed = []

    for j in range(num_cols):
        Cp()
        tj = []
        for i in range(num_rows):
            row = m[i] if i < len(m) else None
            item = row[j] if row and j < len(row) else None
            tj.append(item)
        transposed.append(tj)
    return transposed
