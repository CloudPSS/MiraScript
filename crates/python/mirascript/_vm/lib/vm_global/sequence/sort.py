from __future__ import annotations
from functools import cmp_to_key
import math
from typing_extensions import Callable

from ....._helpers.convert import to_number
from ....._helpers.constants import Uninitialized
from ....operations import Call
from ....types import VmValue, VmAny
from ..._helpers import _expect_array, _expect_callable


def _to_compare(a: int | float, b: int | float) -> int:
    if math.isnan(a):
        a = 0
    if math.isnan(b):
        b = 0
    if a < b:
        return -1
    if a > b:
        return 1
    return 0


def _default_compare(a: VmAny = Uninitialized, b: VmAny = Uninitialized) -> int:
    if a is Uninitialized:
        a = ""
    if b is Uninitialized:
        b = ""
    if isinstance(a, str) and isinstance(b, str):
        if a < b:
            return -1
        elif a > b:
            return 1
        return 0

    if a is b:
        return 0

    return _to_compare(to_number(a, 0), to_number(b, 0))


def _cmp(comparator: VmAny, recovered) -> Callable[[VmValue, VmValue], int]:
    if comparator is None or comparator is Uninitialized:
        return _default_compare
    _expect_callable("comparator", comparator, recovered)

    def compare(a: VmValue, b: VmValue):
        ret = Call(comparator, a, b)
        return _to_compare(to_number(ret), 0)

    return compare


def sort(data=Uninitialized, comparator=Uninitialized):
    data = _expect_array("data", data, None)
    compare = _cmp(comparator, data)
    arr = data.copy()
    arr.sort(key=cmp_to_key(compare))
    return arr


def sort_by(data=Uninitialized, key_fn=Uninitialized, comparator=Uninitialized):
    data = _expect_array("data", data, None)
    key_fn = _expect_callable("key_fn", key_fn, data)
    compare = _cmp(comparator, data)

    arr = []

    l = len(data)

    for i in range(l):
        item = data[i]
        if item is Uninitialized:
            item = None
        key = Call(key_fn, item, i, data)

        arr.append({"o": item, "k": key})

    arr.sort(key=cmp_to_key(lambda a, b: compare(a["k"], b["k"])))
    return [entry["o"] for entry in arr]
