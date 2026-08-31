from __future__ import annotations
import math

from ......_helpers.types import is_vm_array
from .....types import VmConst, VmValue
from .....operations import Cp
from ...._helpers import _expect_array, _expect_integer, _throw_error, _get_numbers
from ...._helpers_utils import _array_len

__all__ = ["zeros", "ones", "identity", "diagonal"]


def _filled(size: tuple[VmValue, ...], value: VmConst) -> VmConst:
    s = _get_numbers(size)
    if len(s) == 0:
        return []

    while len(s) > 0:
        repeat = _array_len(s.pop())
        Cp()
        # 从 MiraScript 语义而言，可以使用同一个引用
        data = [value] * repeat
        value = data
    return value


def zeros(*size):
    return _filled(size, 0)


def ones(*size):
    return _filled(size, 1)


def identity(*size):
    s = _get_numbers(size)
    if len(s) == 0:
        return []
    if len(s) > 2:
        _throw_error("Identity matrix must be 1D or 2D", [])
    if len(s) == 1:
        s = [s[0], s[0]]

    m = _array_len(s[0])
    n = _array_len(s[1])

    result = [[0.0] * n for _ in range(m)]

    for i in range(min(m, n)):
        result[i][i] = 1.0

    return result


def diagonal(vector, k=0):
    _expect_array("vector", vector, [])
    fk = _expect_integer("k", k)
    if math.isnan(fk):
        fk = 0

    if all(is_vm_array(v) for v in vector):
        diag = []
        for i, row in enumerate(vector):
            r = i + fk
            if r < 0:
                continue
            if not row or r >= len(row):
                continue
            diag.append(row[int(r)])

        return diag

    l = len(vector)
    m = _array_len(l - fk if fk < 0 else l)
    n = _array_len(l + fk if fk > 0 else l)

    result = []
    for i in range(m):
        newRow = []
        for j in range(n):
            if j - i == fk:
                ai = i if fk >= 0 else j
                vRow = vector[ai] if ai < len(vector) else None
                newRow.append(vRow)
            else:
                newRow.append(0)
        result.append(newRow)
    return result
