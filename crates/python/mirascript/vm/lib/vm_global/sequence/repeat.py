from ..._helpers import _expect_const, _expect_number, _required
from ..._helpers_utils import _array_len
from mirascript.vm.types.const import Uninitialized


def repeat(data=Uninitialized, times=Uninitialized):
    _expect_const("data", data, [])
    _required("times", times, [])
    n = _array_len(_expect_number("times", times))

    result = []
    for _ in range(n):
        result.append(data)
    return result
