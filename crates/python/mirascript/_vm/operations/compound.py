from __future__ import annotations
import math

from ..._helpers.convert import to_number
from ..._helpers.types import (
    is_vm_array,
    is_vm_extern,
    is_vm_primitive,
    is_vm_record,
    is_vm_wrapper,
)
from ..types import VmAny, VmValue
from ..error import VmError
from .common import AssertInit
from .convert import ToString
from .type_check import Type
from .helpers import Element


def Length(a: VmAny) -> float:
    AssertInit(a)
    if isinstance(a, (list, dict)):
        return float(len(a))
    raise TypeError(f"`Expected array, record or extern, got {Type(a)}")


def Has(obj: VmAny, key: VmAny) -> bool:
    pk = ToString(key)
    if is_vm_wrapper(obj):
        return obj.has(pk)
    if is_vm_record(obj):
        return pk in obj
    if is_vm_array(obj):
        try:
            idx = to_number(key)
            if not idx.is_integer():
                return False
            return 0 <= idx < len(obj)
        except Exception:
            return False
    if is_vm_primitive(obj):
        return False
    AssertInit(obj)
    return False


def Get(obj: VmAny, key: VmAny) -> VmValue:
    if is_vm_array(obj):
        AssertInit(key)
        index = to_number(key, math.nan)
        if not math.isfinite(index):
            return None
        try:
            idx = math.trunc(index)
            return Element(obj[idx])
        except IndexError:
            return None
    if is_vm_primitive(obj):
        return None
    pk = ToString(key)
    if is_vm_wrapper(obj):
        return obj.get(pk)
    if is_vm_record(obj):
        return Element(obj.get(pk, None))
    AssertInit(obj)
    return None


def Set(obj: VmAny, key: VmAny, val: VmAny) -> None:
    pk = ToString(key)
    if not is_vm_extern(obj):
        AssertInit(obj)
        raise VmError(f"`Expected extern object, got {Type(obj)}", None)

    AssertInit(val)
    obj.set(pk, val)
