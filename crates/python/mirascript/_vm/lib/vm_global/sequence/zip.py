from ....._helpers.serialize import display
from ....._helpers.types import is_vm_array
from .entries import entries
from ..._helpers import _throw_error
from ....operations.cp import Cp


def zip(data):
    ets = entries(data)
    length = 0
    for el in ets:
        key = el["0"]
        arr = el["1"]
        if not is_vm_array(arr):
            raise _throw_error(
                f"data[{display(key)}] is not an array: {display(arr)}", None
            )

        length = max(length, len(arr))

    if length == 0:
        return []

    result = []
    is_arr = is_vm_array(data)

    for i in range(length):
        Cp()
        obj = [] if is_arr else {}
        for el in ets:
            key = el["0"]
            arr = el["1"]
            val = arr[i] if i < len(arr) else None
            if is_arr:
                obj.append(val)
            else:
                obj[key] = val
        result.append(obj)

    return result
