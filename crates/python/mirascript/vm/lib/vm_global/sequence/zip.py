from typing import Any, Callable, Dict, List, TypeVar, Union

from ..._helpers import VmLib, VmLibOption, throw_error
from ....types import is_vm_array
from .entries import entries
from .....subtle import serialize

T = TypeVar('T', List[Any], Dict[Any, Any])

def _zip(*args) -> List[Union[List[Any], Dict[Any, Any]]]:
    if len(args) == 0:
        throw_error('zip requires at least one argument', None)
    if len(args) == 1:
        data = args[0]
    else:
        data = list(args)
    
    ets = entries(data)
    length = 0
    for key, arr in ets:
        if not is_vm_array(arr):
            throw_error(f"data[{serialize(key)}] is not an array",None)
        length = max(length, len(arr))
    if length == 0:
        return []
    result = []
    is_arr = is_vm_array(data)
    for i in range(length):
        obj = [] if is_arr else {}
        for key, arr in ets:
            value = arr[i] if i < len(arr) else None
            if isinstance(obj, list):
                obj.append(value)
            else:
                obj[key] = value
        result.append(obj)
    return result

zip = VmLib(
    _zip,
    VmLibOption(
        summary='将多个数组按索引打包成一个数组',
        params= { "data": '要打包的数组或记录' },
        paramsType= { "data": 'array | record' },
        returnsType= 'array',
    ),
)