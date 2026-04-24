from typing import Any, Callable, Dict, List, TypeVar, Union

from ....types.checker import is_vm_array
from .entries import entries
from ..._helpers import throw_error
from ....helpers import Cp


def zip(data):
    ets = entries(data)
    l = 0
    for el in ets:
        key = el["0"]
        arr = el["1"]
        if not is_vm_array(arr):
            raise throw_error(f"zip expected array but got {type(arr)}", None)

        l = max(l, len(arr))

    if l == 0:
        return []

    result = []
    isArr = is_vm_array(data)

    for i in range(l):
        Cp()
        obj = {}
        for el in ets:
            key = el["0"]
            arr = el["1"]
            val = arr[i] if i < len(arr) else None
            if isArr:
                obj[len(obj)] = val
            else:
                obj[key] = val
        result.append(obj if not isArr else list(obj.values()))

    return result
