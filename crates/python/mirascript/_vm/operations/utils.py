import math

from ..._helpers.checker import is_number
from ..._helpers.types import is_vm_array, is_vm_record, is_vm_wrapper
from ..types.types import VmValue


def overload_number_string(a: VmValue, b: VmValue) -> bool:
    if is_number(a) or is_number(b):
        return True
    if isinstance(a, str) or isinstance(b, str):
        return False
    return True


def is_same(a: VmValue, b: VmValue) -> bool:
    if is_number(a) and is_number(b):
        return a == b or (math.isnan(a) and math.isnan(b))
    if a is b:
        return True
    if type(a) is not type(b):
        return False
    if isinstance(a, (str, bool)):
        return a == b
    if is_vm_wrapper(a):
        return a.same(b)
    if is_vm_wrapper(b):
        return b.same(a)
    if is_vm_array(a) and is_vm_array(b):
        if len(a) != len(b):
            return False
        for x, y in zip(a, b):
            if not is_same(x, y):
                return False
        return True
    if is_vm_record(a) and is_vm_record(b):
        if set(a.keys()) != set(b.keys()):
            return False
        for key in a.keys():
            if not is_same(a[key], b[key]):
                return False
        return True
    return False
