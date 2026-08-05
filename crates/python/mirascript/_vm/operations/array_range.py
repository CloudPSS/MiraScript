from __future__ import annotations
import math

from ..._helpers.constants import VM_ARRAY_MAX_LENGTH
from ..types import VmAny
from .convert import ToNumber


def _make_array(start: VmAny, end: VmAny, exclusive: bool) -> list[float]:
    s = ToNumber(start)
    e = ToNumber(end)
    if not math.isfinite(s) or not math.isfinite(e) or s > e:
        return []
    n = math.ceil(e - s) if exclusive else math.floor((e - s) + 1.0)
    if n > VM_ARRAY_MAX_LENGTH:
        raise RuntimeError(
            f"Array length exceeds maximum limit of {VM_ARRAY_MAX_LENGTH}"
        )
    arr: list[float] = [0.0] * n
    for i in range(n):
        arr[i] = s + i
    return arr


def ArrayRange(start: VmAny, end: VmAny) -> list[float]:
    return _make_array(start, end, exclusive=False)


def ArrayRangeExclusive(start: VmAny, end: VmAny) -> list[float]:
    return _make_array(start, end, exclusive=True)
