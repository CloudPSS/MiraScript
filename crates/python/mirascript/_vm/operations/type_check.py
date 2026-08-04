from __future__ import annotations

from ..._helpers.constants import Uninitialized
from ..._helpers.types import get_vm_type, is_vm_array, is_vm_record
from ..._helpers.checker import is_number
from ..error import VmError
from ..types import VmAny
from .common import AssertInit


def Type(val: VmAny):
    if val is Uninitialized or val is None:
        return "nil"
    return get_vm_type(val)


def IsBoolean(val: VmAny) -> bool:
    AssertInit(val)
    return isinstance(val, bool)


def IsNumber(val: VmAny) -> bool:
    AssertInit(val)
    return is_number(val)


def IsString(val: VmAny) -> bool:
    AssertInit(val)
    return isinstance(val, str)


def IsRecord(val: VmAny) -> bool:
    AssertInit(val)
    return is_vm_record(val)


def IsArray(val: VmAny) -> bool:
    AssertInit(val)
    return is_vm_array(val)


def AssertNonNil(val: VmAny) -> None:
    AssertInit(val)
    if val is not None:
        return
    raise VmError("Expected non-nil value", None)
