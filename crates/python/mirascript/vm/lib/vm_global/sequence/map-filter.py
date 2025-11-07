from ..._helpers import VmLib, expect_callable, expect_const, map_impl
from ....types import is_vm_const

def map_impl_wrapped(data, fn_name, fn, mapper):
    expect_const('data', data, None)
    expect_callable(fn_name, fn, data)
    def wrapped(value, index, data_):
        ret = mapper(fn, value, index, data_)
        if ret is None or is_vm_const(ret):
            return ret
        return None
    return map_impl(data, wrapped)

def map_(data, f):
    return map_impl_wrapped(data, 'f', f, lambda fn, value, key, data_: fn(value, key, data_))

def filter_(data, predicate):
    return map_impl_wrapped(
        data, 'predicate', predicate,
        lambda fn, value, key, data_: value if bool(fn(value, key, data_)) else None
    )

def filter_map(data, f):
    return map_impl_wrapped(
        data, 'f', f,
        lambda fn, value, key, data_: fn(value, key, data_) or None
    )

map = VmLib(
    map_,
    {
        "summary": "对数组或记录中的每个元素应用函数，并返回结果",
        "params": {
            "data": "要映射的数组或记录",
            "f": "应用于每个元素的函数",
        },
        "paramsType": {
            "data": "array | record",
            "f": "fn(value: any, key: number | string | nil, input: type(data)) -> any",
        },
        "returnsType": "type(data)",
    },
)

filter = VmLib(
    filter_,
    {
        "summary": "过滤数组或记录中的元素，返回满足条件的元素",
        "params": {
            "data": "要过滤的数组或记录",
            "predicate": "用于测试每个元素的函数，返回 true 或 false",
        },
        "paramsType": {
            "data": "array | record",
            "predicate": "fn(value: any, key: number | string | nil, input: type(data)) -> boolean",
        },
        "returnsType": "type(data)",
    },
)

filter_map = VmLib(
    filter_map,
    {
        "summary": "对数组或记录中的每个元素应用函数，并返回非 nil 的结果",
        "params": {
            "data": "要映射的数组或记录",
            "f": "应用于每个元素的函数，返回 nil 或非 nil 的值",
        },
        "paramsType": {
            "data": "array | record",
            "f": "fn(value: any, key: number | string | nil, input: type(data)) -> any | nil",
        },
        "returnsType": "type(data)",
    },
)