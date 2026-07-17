from __future__ import annotations
from typing_extensions import cast

from ....._helpers.serialize import display
from ....._helpers.types import is_vm_array
from ....types.types import VmAny, VmArray, VmRecord
from ....operations.cp import Cp
from ..._helpers import _throw_error
from .entries import entries


def zip(data: VmAny = None) -> VmArray:
    ets = entries(data)
    length = 0
    for el in ets:
        key = el["0"]
        arr = el["1"]
        if not is_vm_array(arr):
            return _throw_error(
                f"data[{display(key)}] is not an array: {display(arr)}", None
            )

        length = max(length, len(arr))

    if length == 0:
        return []

    result: VmArray = []
    is_arr = is_vm_array(data)

    for i in range(length):
        Cp()
        obj: VmArray | VmRecord = [] if is_arr else {}
        for el in ets:
            key = el["0"]
            arr = el["1"]
            val = arr[i] if i < len(arr) else None
            if is_arr:
                cast(VmArray, obj).append(val)
            else:
                cast(VmRecord, obj)[key] = val
        result.append(obj)

    return result
