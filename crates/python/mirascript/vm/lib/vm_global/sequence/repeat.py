from ..._helpers import  expect_callable, expect_const, map_vm as map_impl,required
from ..._helpers_utils import array_len
from ....types import is_vm_const
from ....operations import Call_,ToNumber_
from mirascript.vm.types.const import Uninitialized


def repeat(data=Uninitialized, times=Uninitialized):
    expect_const('data', data, [])
    required('times', times, [])
    n = array_len(ToNumber_(times))
    
    result = []
    for _ in range(n):
        result.append(data)
    return result