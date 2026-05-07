from .....helpers.convert.to_boolean import toBoolean
from ....operations import Call, is_same
from ....types import VmValue
from ..._helpers import _expect_array, _expect_callable
from ....types.types import Uninitialized


def default_equal(a=None, b=None):
    if a is Uninitialized:
        a = None
    if b is Uninitialized:
        b = None
    return is_same(a, b)


def eq(equaler, recovered):
    if equaler is None or equaler is Uninitialized:
        return default_equal
    _expect_callable("equal", equaler, recovered)

    def equal(a: VmValue, b: VmValue):
        ret = Call(equaler, a, b)
        return toBoolean(ret)

    return equal


def unique(data=Uninitialized, equal=Uninitialized):
    _expect_array("data", data, None)
    e = eq(equal, data)
    arr = []
    for item in data:
        found = False
        for unique_item in arr:
            if e(item, unique_item):
                found = True
                break
        if not found:
            arr.append(item)
    return arr


def unique_by(data=Uninitialized, key_fn=Uninitialized, equal=Uninitialized):
    _expect_array("data", data, None)
    _expect_callable("key_fn", key_fn, data)
    e = eq(equal, data)

    arr = []
    keys = []

    l = len(data)

    for i in range(l):
        item = data[i]
        key = Call(key_fn, item, i, data)
        found = False
        for unique_key in keys:
            if e(key, unique_key):
                found = True
                break
        if not found:
            arr.append(item)
            keys.append(key)
    return arr
