import math

from ...helpers.constants import VM_ARRAY_MAX_LENGTH
from ..types.types import VmArray
from .convert import ToNumber


def _assert_array_length(length: float) -> None:
    if length > VM_ARRAY_MAX_LENGTH:
        raise RuntimeError(
            f"Array length exceeds maximum limit of {VM_ARRAY_MAX_LENGTH}"
        )


def _is_empty_range(start: float, end: float) -> bool:
    return not math.isfinite(start) or not math.isfinite(end) or start > end


def ArrayRange(start, end) -> VmArray:
    s = ToNumber(start)
    e = ToNumber(end)
    if _is_empty_range(s, e):
        return []
    _assert_array_length(e - s + 1)
    arr: VmArray = []
    i = s
    while i <= e:
        arr.append(i)
        i += 1.0
    return arr


def ArrayRangeExclusive(start, end) -> VmArray:
    s = ToNumber(start)
    e = ToNumber(end)
    if _is_empty_range(s, e):
        return []
    _assert_array_length(e - s)
    arr: VmArray = []
    i = s
    while i < e:
        arr.append(i)
        i += 1.0
    return arr
