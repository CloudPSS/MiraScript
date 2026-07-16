from ..._helpers import _expect_callable, _expect_integer_range
from ....operations import Call, Get, Element
from ....._helpers.constants import VM_ARRAY_MAX_LENGTH, Uninitialized
from ....._helpers.convert import to_string


def new_record(size=Uninitialized, generator=Uninitialized):
    n = int(_expect_integer_range("size", size, 0, float("inf")))
    _expect_callable("generator", generator, None)
    record = {}
    for i in range(n):
        entry = Call(generator, i)
        if entry is None:
            continue
        key = Get(entry, 0)
        value = Get(entry, 1)
        record[to_string(key)] = Element(value)
    return record


def new_array(length=Uninitialized, generator=Uninitialized):
    n = int(_expect_integer_range("length", length, 0, VM_ARRAY_MAX_LENGTH))
    _expect_callable("generator", generator, None)
    return [Element(Call(generator, i)) for i in range(n)]
