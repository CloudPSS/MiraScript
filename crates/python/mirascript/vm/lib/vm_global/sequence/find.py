from ..._helpers import VmLib, VmLibOption, expect_array_or_record, expect_callable
from ....types import is_vm_array

def _find(data, predicate):
    expect_array_or_record('data', data, None)
    expect_callable('predicate', predicate, data)

    def p(value, key, data):
        ret = predicate(value, key, data)
        return bool(ret)

    if is_vm_array(data):
        for i, item in enumerate(data):
            value = item if item is not None else None
            if p(value, i, data):
                return (i, value)
        return None
    else:
        for key, v in data.items():
            value = v if v is not None else None
            if p(value, key, data):
                return (key, value)
        return None

find = VmLib(
    _find,
    VmLibOption(
        summary= "查找数组或记录中的键值对，返回第一个满足条件的键值对",
        params= {
            "data": "查的数组或记录",
            "predicate": "用于测试每个键值对的函数，返回 true 或 false",
        },
        paramsType= {
            "data": "array | record",
            "predicate": "fn(value: any, key: number | string | nil, input: type(data)) -> boolean",
        },
        returnsType= "(string | number, any) | nil",
    ),
)