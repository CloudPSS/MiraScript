from __future__ import annotations
import math

from ..._helpers.checker import is_number
from ..._helpers.types import is_vm_wrapper, get_vm_type
from ..types import VmAny, VmArray, VmRecord


def overload_number_string(a: VmAny, b: VmAny) -> bool:
    if is_number(a) or is_number(b):
        return True
    if isinstance(a, str) or isinstance(b, str):
        return False
    return True


def _is_array_same(a: VmArray, b: VmArray) -> bool:
    if len(a) != len(b):
        return False
    for x, y in zip(a, b):
        if not is_same(x, y):
            return False
    return True


def _is_record_same(a: VmRecord, b: VmRecord) -> bool:
    if set(a.keys()) != set(b.keys()):
        return False
    for key in a.keys():
        if not is_same(a[key], b[key]):
            return False
    return True


def is_same(a: VmAny, b: VmAny) -> bool:
    if is_number(a) and is_number(b):
        return a == b or (math.isnan(a) and math.isnan(b))
    if a is b:
        return True
    ta = get_vm_type(a)
    tb = get_vm_type(b)
    if ta != tb:
        return False
    if ta in ("string", "boolean"):
        return a == b
    if ta == "array":
        return _is_array_same(a, b)  # type: ignore
    if ta == "record":
        return _is_record_same(a, b)  # type: ignore
    if is_vm_wrapper(a):
        return a.same(b)
    if is_vm_wrapper(b):
        return b.same(a)
    return False
