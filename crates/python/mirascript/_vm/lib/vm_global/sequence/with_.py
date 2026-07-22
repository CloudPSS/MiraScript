import math

from ....._helpers.checker import is_number
from ....._helpers.types import is_vm_array, is_vm_record
from ....._helpers.convert import to_number, to_string
from ....._helpers.constants import Uninitialized, VM_ARRAY_MAX_LENGTH
from ....operations.helpers import Element
from ....types import VmConst
from ..._helpers import (
    _expect_array_or_record,
    _throw_error,
    _expect_const,
)


def _arr_index(index: VmConst) -> int:
    num = to_number(index, math.nan)
    if not math.isfinite(num) or num < 0:
        _throw_error("Array index must be a non-negative integer", index)
    idx = math.trunc(num)
    if idx >= VM_ARRAY_MAX_LENGTH:
        _throw_error(
            f"Array index exceeds maximum limit of {VM_ARRAY_MAX_LENGTH}", index
        )
    return idx


def _is_arr_index(key):
    if not is_number(key):
        return False
    return key.is_integer() and key >= 0 and key <= VM_ARRAY_MAX_LENGTH


def _with_inner(obj, key, key_index, value):
    if key_index >= len(key):
        return value

    k = key[key_index]
    result = None
    if is_vm_array(obj):
        result = obj.copy()
    elif is_vm_record(obj):
        result = obj.copy()
    elif _is_arr_index(k):
        result = []
    else:
        result = {}

    if is_vm_array(result):
        index = int(_arr_index(k))
        while index >= len(result):
            result.append(None)
        result[index] = _with_inner(result[index], key, key_index + 1, value)
    else:
        key_str = to_string(k)
        result[key_str] = _with_inner(
            result.get(key_str, None), key, key_index + 1, value
        )
    return result


def _normalize_entries(data, entries):
    if len(entries) % 2 != 0:
        _throw_error(
            "with_ function requires even number of arguments as key-value pairs", data
        )

    entryData = []

    for i in range(0, len(entries), 2):
        key = entries[i]
        _expect_const("key", key, data)
        if key is None or key is Uninitialized:
            continue

        if is_vm_array(key):
            if len(key) == 0 or None in key or Uninitialized in key:
                continue
            if len(key) == 1:
                key = key[0]

        value = entries[i + 1]
        entryData.append((key, Element(value)))

    return entryData


def with_(data=Uninitialized, *args):
    data = _expect_array_or_record("data", data, [])
    if len(args) == 0:
        return data

    entryData = _normalize_entries(data, args)
    if is_vm_array(data):
        result = data.copy()
        for key, element in entryData:
            index = 0
            val = None

            if is_vm_array(key):
                index = _arr_index(key[0])
                if index < 0:
                    continue
                val = _with_inner(
                    result[int(index)] if index < len(result) else None, key, 1, element
                )
            else:
                index = _arr_index(key)
                if index < 0:
                    continue
                val = element
            while index >= len(result):
                result.append(None)
            result[int(index)] = val
        return result
    else:
        result = data.copy()
        for key, element in entryData:
            if is_vm_array(key):
                firstKey = key[0]
                prop = to_string(firstKey)
                val = _with_inner(result.get(prop, None), key, 1, element)
            else:
                prop = to_string(key)
                val = element
            result[prop] = val
        return result
