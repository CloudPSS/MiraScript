import math

from ...helpers.types import is_vm_array
from ..error import VmError
from ..types.types import VmArray
from .utils import is_safe_integer
from .common import AssertInit
from .convert import ToNumber
from .type_check import Type


def _slice_core(value: VmArray, start: float, end: float, exclusive: bool) -> VmArray:
    length = len(value)

    if math.isnan(start) or (math.isinf(start) and start < 0):
        start = 0
    elif math.isinf(start):
        return []
    elif start < 0:
        start = length + start

    if math.isnan(end) or (math.isinf(end) and end > 0):
        end = length if exclusive else length - 1
    elif math.isinf(end):
        return []
    elif end < 0:
        end = length + end

    start = math.ceil(start)
    if exclusive or not is_safe_integer(end):
        end = math.ceil(end)
    else:
        end = math.trunc(end + 1)
    return value[start:end]


def Slice(a, start, end):
    AssertInit(a)
    if not is_vm_array(a):
        raise VmError(f"`Expected array, got {Type(a)}", [])
    s = ToNumber(start) if start is not None else 0
    e = ToNumber(end) if end is not None else len(a) - 1
    return _slice_core(a, s, e, False)


def SliceExclusive(a, start, end):
    AssertInit(a)
    if not is_vm_array(a):
        raise VmError(f"`Expected array, got {Type(a)}", [])
    s = ToNumber(start) if start is not None else 0
    e = ToNumber(end) if end is not None else len(a)
    return _slice_core(a, s, e, True)
