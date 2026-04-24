from mirascript.helpers.convert.to_number import toNumber
from ....operations import Call_
from ....types import VmValue
from ..._helpers import (
    _expect_array_or_record,
    _expect_compound,
    _expect_array,
    _expect_callable,
)
from mirascript.vm.types.const import Uninitialized
from functools import cmp_to_key
import math


def default_compare(a=Uninitialized, b=Uninitialized):
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
    a_num = toNumber(a, 0)
    if math.isnan(a_num):
        a_num = 0
    b_num = toNumber(b, 0)
    if math.isnan(b_num):
        b_num = 0
    return a_num - b_num


def cmp(comparator, recovered):
    if comparator is None or comparator is Uninitialized:
        return default_compare
    _expect_callable("comparator", comparator, recovered)

    def compare(a: VmValue, b: VmValue):
        ret = Call_(comparator, a, b)
        return toNumber(ret, 0)

    return compare


def sort(data=Uninitialized, comparator=Uninitialized):
    _expect_array("data", data, data)
    compare = cmp(comparator, data)
    arr = []
    for item in data:
        arr.append(item)
    arr.sort(key=cmp_to_key(compare))
    return arr


def sort_by(data=Uninitialized, key_fn=Uninitialized, comparator=Uninitialized):
    _expect_array("data", data, data)
    _expect_callable("key_fn", key_fn, data)
    compare = cmp(comparator, data)

    arr = []

    l = len(data)

    for i in range(l):
        item = data[i]
        if item is Uninitialized:
            item = None
        key = Call_(key_fn, item, i, data)

        arr.append({"o": item, "k": key})

    arr.sort(key=cmp_to_key(lambda a, b: compare(a["k"], b["k"])))
    return [entry["o"] for entry in arr]
