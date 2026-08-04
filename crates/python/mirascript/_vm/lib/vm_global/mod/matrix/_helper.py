from __future__ import annotations

from ......_helpers.convert import to_number
from ......_helpers.types import is_vm_array
from .....types import VmAny, VmValue


def _size(matrix: VmValue) -> tuple[int, ...]:
    if not is_vm_array(matrix):
        return ()
    if len(matrix) == 0:
        return (0,)

    num_rows = len(matrix)
    num_cols = 0

    for row in matrix:
        if is_vm_array(row):
            num_cols = max(num_cols, len(row))
        else:
            return (num_rows,)
    return (num_rows, num_cols)


def _num(v: VmAny) -> float:
    return to_number(v)
