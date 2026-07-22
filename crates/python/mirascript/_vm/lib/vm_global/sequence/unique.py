from __future__ import annotations

from ....._helpers.convert import to_boolean
from ....operations.utils import is_same
from ....operations import Call
from ....types import VmValue
from ..._helpers import _expect_array, _expect_callable
from ....types import Uninitialized, VmAny


def _default_equal(a: VmAny = None, b: VmAny = None) -> bool:
    if a is Uninitialized:
        a = None
    if b is Uninitialized:
        b = None
    return is_same(a, b)


def _eq(equaler, recovered):
    if equaler is None or equaler is Uninitialized:
        return _default_equal
    equaler = _expect_callable("equal", equaler, recovered)

    def equal(a: VmValue, b: VmValue):
        ret = Call(equaler, a, b)
        return to_boolean(ret)

    return equal


def unique(data=Uninitialized, equal=Uninitialized):
    data = _expect_array("data", data, None)
    e = _eq(equal, data)
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
    data = _expect_array("data", data, None)
    key_fn = _expect_callable("key_fn", key_fn, data)
    e = _eq(equal, data)

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
