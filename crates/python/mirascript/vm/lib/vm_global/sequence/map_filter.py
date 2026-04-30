from mirascript.helpers.types import is_vm_const

from ..._helpers import _expect_callable, _expect_const, _map_vm
from ....operations import Call, ToBoolean
from mirascript.vm.types.types import Uninitialized


def _map_impl_wrapped(data, fn_name, fn, mapper):
    _expect_const("data", data, None)
    _expect_callable(fn_name, fn, data)

    def wrapped(value, index, data_):
        ret = mapper(fn, value, index, data_)
        if ret is Uninitialized or is_vm_const(ret):
            return ret
        return None

    return _map_vm(data, wrapped)


def map(data=Uninitialized, f=Uninitialized):
    return _map_impl_wrapped(
        data, "f", f, lambda fn, value, key, data_: Call(fn, *(value, key, data_))
    )


def filter(data=Uninitialized, predicate=Uninitialized):
    return _map_impl_wrapped(
        data,
        "predicate",
        predicate,
        lambda fn, value, key, data_: (
            value if ToBoolean(Call(fn, *(value, key, data_))) else Uninitialized
        ),
    )


def filter_map(data=Uninitialized, f=Uninitialized):
    return _map_impl_wrapped(
        data,
        "f",
        f,
        lambda fn, value, key, data_: Call(fn, *(value, key, data_)) or Uninitialized,
    )
