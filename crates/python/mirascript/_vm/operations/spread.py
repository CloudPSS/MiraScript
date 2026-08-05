from __future__ import annotations
from typing_extensions import Iterable as IterableType, Mapping

from ..._helpers.convert.string import number_to_string
from ..._helpers.types import is_vm_array, is_vm_extern, is_vm_record
from ..types import VmAny, VmConst
from ..error import VmError
from .common import AssertInit
from .type_check import Type


def RecordSpread(record: VmAny) -> Mapping[str, VmConst]:
    AssertInit(record)
    if record is None:
        # Cannot spread None in python
        return {}
    if is_vm_record(record):
        return record
    if is_vm_array(record):
        return {number_to_string(i): record[i] for i in range(len(record))}
    if is_vm_extern(record):
        return {}

    raise VmError(f"`Expected record, extern or nil, got {Type(record)}", None)


def ArraySpread(array: VmAny) -> IterableType[VmConst]:
    AssertInit(array)
    if array is None:
        return []
    if is_vm_array(array):
        return array
    if is_vm_extern(array):
        pass

    raise VmError(f"`Expected array, iterable extern or nil, got {Type(array)}", None)
