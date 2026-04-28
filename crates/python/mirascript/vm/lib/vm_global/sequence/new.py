from ..._helpers import _expect_callable, _expect_integer_range
from ....operations import Call_, ToBoolean_, Get_
from ....helpers import Element
from mirascript.vm.types.const import VM_ARRAY_MAX_LENGTH, Uninitialized
from mirascript.helpers.convert.to_string import toString


def new_record(size=Uninitialized, generator=Uninitialized):
    n = int(_expect_integer_range("size", size, 0, float("inf")))
    _expect_callable("generator", generator, None)
    record = {}
    for i in range(n):
        entry = Call_(generator, i)
        if entry is None:
            continue
        key = Get_(entry, 0)
        value = Get_(entry, 1)
        record[toString(key)] = Element(value)
    return record


def new_array(length=Uninitialized, generator=Uninitialized):
    n = int(_expect_integer_range("length", length, 0, VM_ARRAY_MAX_LENGTH))
    _expect_callable("generator", generator, None)
    return [Element(Call_(generator, i)) for i in range(n)]
