import math

from ..._helpers.constants import VM_ARRAY_MAX_LENGTH
from ..types.types import VmAny, VmArray
from .convert import ToNumber


def _assert_array_length(length: int) -> None:
    if length > VM_ARRAY_MAX_LENGTH:
        raise RuntimeError(
            f"Array length exceeds maximum limit of {VM_ARRAY_MAX_LENGTH}"
        )


def _is_empty_range(start: float, end: float) -> bool:
    return not math.isfinite(start) or not math.isfinite(end) or start > end


def ArrayRange(start: VmAny, end: VmAny) -> VmArray:
    s = ToNumber(start)
    e = ToNumber(end)
    if _is_empty_range(s, e):
        return []
    n = math.floor((e - s) + 1.0)
    _assert_array_length(n)
    arr: VmArray = [0.0] * n
    for i in range(n):
        arr[i] = s + i
    return arr


def ArrayRangeExclusive(start: VmAny, end: VmAny) -> VmArray:
    s = ToNumber(start)
    e = ToNumber(end)
    if _is_empty_range(s, e):
        return []
    n = math.ceil(e - s)
    _assert_array_length(n)
    arr: VmArray = [0.0] * n
    for i in range(n):
        arr[i] = s + i
    return arr
