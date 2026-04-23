from ..._helpers import expect_const, expect_number, map_vm as map_impl, required
from ..._helpers_utils import array_len
from mirascript.vm.types.const import Uninitialized


def repeat(data=Uninitialized, times=Uninitialized):
    expect_const("data", data, [])
    required("times", times, [])
    n = array_len(expect_number("times", times))

    result = []
    for _ in range(n):
        result.append(data)
    return result
