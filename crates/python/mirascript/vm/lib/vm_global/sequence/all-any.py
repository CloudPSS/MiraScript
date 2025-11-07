from ..._helpers import VmLib, throw_error
from ....types import is_vm_array

def _all(data, fn):
    if not is_vm_array(data):
        throw_error("data is not an array", None)
    return all(fn(item, i) for i, item in enumerate(data))

def _any(data, fn):
    if not is_vm_array(data):
        throw_error("data is not an array", None)
    return any(fn(item, i) for i, item in enumerate(data))

all_ = VmLib(
    _all,
    {
        "summary": "判断数组所有元素是否都满足条件",
        "params": {"data": "要判断的数组", "fn": "判断函数"},
        "paramsType": {"data": "array", "fn": "function"},
        "returnsType": "boolean",
    },
)

any_ = VmLib(
    _any,
    {
        "summary": "判断数组是否有元素满足条件",
        "params": {"data": "要判断的数组", "fn": "判断函数"},
        "paramsType": {"data": "array", "fn": "function"},
        "returnsType": "boolean",
    },
)