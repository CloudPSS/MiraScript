from __future__ import annotations
from typing_extensions import Sequence

from ..._helpers.types import is_vm_record
from ..._helpers.convert import to_string
from ..types import VmAny, VmRecord
from .common import AssertInit


def Omit(value: VmAny, omitted: Sequence[float | str | int]) -> VmRecord:
    AssertInit(value)
    if not is_vm_record(value):
        return {}
    result: VmRecord = {}
    valueKeys = value.keys()
    omittedSet = set([to_string(x) for x in omitted])
    for key in valueKeys:
        if key not in omittedSet:
            result[key] = value[key]
    return result


def Pick(value: VmAny, picked: Sequence[float | str | int]) -> VmRecord:
    AssertInit(value)
    if not is_vm_record(value):
        return {}
    result: VmRecord = {}
    for key in picked:
        k = to_string(key)
        if k in value:
            result[k] = value[k]

    return result
