import math

from ..._helpers.convert import to_number, to_string
from ..._helpers.convert.to_string import number_to_string
from ..._helpers.types import (
    is_vm_array,
    is_vm_extern,
    is_vm_primitive,
    is_vm_record,
    is_vm_wrapper,
)
from ..types import VmAny, VmRecord, VmValue
from ..error import VmError
from .common import AssertInit
from .convert import ToString
from .type_check import Type
from .helpers import Element
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


def Length(a: VmAny) -> float:
    AssertInit(a)
    if isinstance(a, (str, list, dict)):
        return float(len(a))
    raise TypeError(f"`Expected array, string or record, got {Type(a)}")


def Omit(a: VmAny, b: VmAny) -> VmRecord:
    AssertInit(a)
    if not is_vm_record(a):
        return {}
    result = {}

    valueKeys = a.keys()
    omittedSet = set([ToString(x) for x in b])
    for key in valueKeys:
        if key not in omittedSet:
            result[key] = a[key]
    return result


def Pick(a, b):
    AssertInit(a)
    if not is_vm_record(a):
        return {}
    result = {}
    for key in b:
        k = ToString(key)
        if k in a:
            result[k] = a[k]

    return result


def Has(obj, key):
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


def Set(obj, key, val):
    pk = ToString(key)
    if not is_vm_extern(obj):
        AssertInit(obj)
        raise VmError(f"`Expected extern object, got {Type(obj)}", None)

    AssertInit(val)
    obj.set(pk, val)


def Iterable(value):
    AssertInit(value)
    if is_vm_wrapper(value):
        return value.keys()
    if is_vm_array(value):
        return value
    if is_vm_record(value):
        return value.keys()
    raise VmError(f"`Value is not iterable {Type(value)}", None)


def RecordSpread(record):
    AssertInit(record)
    if record is None:
        return {}
    if is_vm_record(record):
        return record
    if is_vm_array(record):
        return {number_to_string(i): record[i] for i in range(len(record))}
    if is_vm_extern(record):
        return {}

    raise VmError(f"`Expected record, extern or nil, got {Type(record)}", None)


def ArraySpread(array):
    AssertInit(array)
    if array is None:
        return []
    if is_vm_array(array):
        return array
    if is_vm_extern(array):
        pass

    raise VmError(f"`Expected array, iterable extern or nil, got {Type(array)}", None)
