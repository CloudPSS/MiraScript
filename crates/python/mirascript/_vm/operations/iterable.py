from __future__ import annotations
from typing_extensions import Iterable as IterableType

from ..._helpers.convert import to_string
from ..._helpers.types import is_vm_array, is_vm_primitive, is_vm_record, is_vm_wrapper
from ..types import VmAny, VmValue
from ..error import VmError
from .common import AssertInit
from .type_check import Type
from .utils import is_same


def In(value: VmAny, iterable: VmAny) -> bool:
    AssertInit(value)
    if is_vm_array(iterable):
        if value is None:
            return value in iterable

        for item in iterable:
            if is_same(value, item):
                return True

        return False
    if is_vm_primitive(iterable):
        return False
    pk = to_string(value)
    if is_vm_record(iterable):
        return pk in iterable
    if is_vm_wrapper(iterable):
        return iterable.has(pk)

    AssertInit(iterable)
    return False


def Iterable(value: VmAny) -> IterableType[VmValue]:
    AssertInit(value)
    if is_vm_wrapper(value):
        return value.keys()
    if is_vm_array(value):
        return value
    if is_vm_record(value):
        return value.keys()
    raise VmError(f"`Value is not iterable {Type(value)}", None)
