import math

from ..._helpers.checker import is_safe_integer
from ..._helpers.serialize import display
from ..._helpers.convert import to_number
from ..._helpers.types import is_vm_array
from ..error import VmError
from ..types import VmAny, VmArray
from .common import AssertInit


def _slice_core(value: VmAny, start: VmAny, end: VmAny, exclusive: bool) -> VmArray:
    AssertInit(value)
    AssertInit(start)
    AssertInit(end)

    if not is_vm_array(value):
        raise VmError(f"`Expected array, got {display(value)}`", [])
    length = len(value)
    s = to_number(start) if start is not None else 0
    e = to_number(end) if end is not None else length - (0 if exclusive else 1)

    if math.isnan(s) or (math.isinf(s) and s < 0):
        s = 0
    elif math.isinf(s):
        return []
    elif s < 0:
        s = length + s

    if math.isnan(e) or (math.isinf(e) and e > 0):
        e = length if exclusive else length - 1
    elif math.isinf(e):
        return []
    elif e < 0:
        e = length + e

    s = math.ceil(s)
    if exclusive or not is_safe_integer(e):
        e = math.ceil(e)
    else:
        e = math.trunc(e + 1)
    return value[s:e]


def Slice(value: VmAny, start: VmAny, end: VmAny) -> VmArray:
    return _slice_core(value, start, end, False)


def SliceExclusive(value: VmAny, start: VmAny, end: VmAny) -> VmArray:
    return _slice_core(value, start, end, True)
