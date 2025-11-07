from math import trunc,isfinite,inf,nan
from ....helpers import Element
from ....types import is_vm_array,VmConst
from ....operations import ToNumber_,ToString_
from ..._helpers import VmLib,expect_array_or_record,throw_error


def _with(data, *entries):
    expect_array_or_record('data', data,data)
    if len(entries) % 2 != 0:
        throw_error('with', '参数错误，键值对成对出现')
    is_array = is_vm_array(data)
    
    if is_array:
        result = []
        for i in range(0, len(entries), 2):
            index = trunc(ToNumber_(entries[i]))
            if index <0 or not isfinite(index) or index >= 2**53 - 1:
                continue
            val = entries[i + 1]
            while index >= len(result):
                result.append(None)
            result[index] = Element(val)
        return result
    else:
        result = {}
        for i in range(0, len(entries), 2):
            key = ToString_(entries[i])
            val = entries[i + 1]
            result[key] = Element(val)
        return result

with_ = VmLib(
    _with,
    {
        "summary": '在数组或记录中设置多个键值对',
        "params": { "data": '要设置的数组或记录', "..entries": '要设置的键值对，成对出现' },
        "paramsType": { "data": 'array | record', "..entries": '[..[string | number, any][]]' },
        "returnsType": 'type(data)',
    },
)