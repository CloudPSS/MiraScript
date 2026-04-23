from ..._helpers import  expect_callable, expect_const, map_vm as map_impl
from ....types import is_vm_const
from ....operations import Call_,ToBoolean_
from mirascript.vm.types.const import Uninitialized
def map_impl_wrapped(data, fn_name, fn, mapper):
    expect_const('data', data, None)
    expect_callable(fn_name, fn, data)
    def wrapped(value, index, data_):
        ret = mapper(fn, value, index, data_)
        if ret is Uninitialized or is_vm_const(ret):
            return ret
        return None
    return map_impl(data, wrapped)

def map(data=Uninitialized, f=Uninitialized):
    return map_impl_wrapped(data, 'f', f, lambda fn, value, key, data_: Call_(fn,*(value, key, data_)))

def filter(data=Uninitialized, predicate=Uninitialized):
    return map_impl_wrapped(
        data, 'predicate', predicate,
        lambda fn, value, key, data_: value if ToBoolean_(Call_(fn,*(value, key, data_))) else Uninitialized
    )

def filter_map(data=Uninitialized, f=Uninitialized):
    return map_impl_wrapped(
        data, 'f', f,
        lambda fn, value, key, data_: Call_(fn,*(value, key, data_)) or Uninitialized
    )

