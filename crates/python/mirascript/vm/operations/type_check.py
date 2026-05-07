from ...helpers.constants import Uninitialized
from ...helpers.types import (
    is_vm_array,
    is_vm_extern,
    is_vm_function,
    is_vm_module,
    is_vm_record,
)
from ...vm.error import VmError
from .common import AssertInit
from .utils import is_number


def Type(val):
    if val is Uninitialized or val is None:
        return "nil"
    if isinstance(val, bool):
        return "boolean"
    if isinstance(val, (int, float)):
        return "number"
    if isinstance(val, str):
        return "string"
    if is_vm_array(val):
        return "array"
    if is_vm_record(val):
        return "record"
    if is_vm_extern(val):
        return "extern"
    if is_vm_module(val):
        return "module"
    if is_vm_function(val):
        return "function"
    return type(val).__name__


def IsBoolean(val):
    AssertInit(val)
    return isinstance(val, bool)


def IsNumber(val):
    AssertInit(val)
    return is_number(val)


def IsString(val):
    AssertInit(val)
    return isinstance(val, str)


def IsRecord(val):
    AssertInit(val)
    return is_vm_record(val)


def IsArray(val):
    AssertInit(val)
    return is_vm_array(val)


def AssertNonNil(val):
    AssertInit(val)
    if val is not None:
        return
    raise VmError("Expected non-nil value", None)
