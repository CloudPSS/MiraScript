from ..._helpers import  expect_callable, expect_const, map_vm as map_impl
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

def map(data, f):
    return map_impl_wrapped(data, 'f', f, lambda fn, value, key, data_: fn(value, key, data_))

def filter(data, predicate):
    return map_impl_wrapped(
        data, 'predicate', predicate,
        lambda fn, value, key, data_: value if bool(fn(*[value, key, data_])) else None
    )

def filter_map(data, f):
    return map_impl_wrapped(
        data, 'f', f,
        lambda fn, value, key, data_: fn(value, key, data_) or None
    )

