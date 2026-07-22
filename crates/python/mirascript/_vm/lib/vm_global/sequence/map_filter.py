from __future__ import annotations
from typing_extensions import Callable

from ....._helpers.types import is_vm_const
from ....types import Uninitialized, VmAny, VmFunction
from ..._helpers import _expect_callable, _expect_const, _map_vm
from ....operations import Call, ToBoolean


def _map_impl_wrapped(
    data: VmAny,
    fn_name: str,
    fn: VmAny,
    mapper: Callable[[VmFunction, VmAny, VmAny, VmAny], VmAny],
) -> VmAny:
    data = _expect_const("data", data, None)
    fn = _expect_callable(fn_name, fn, data)

    def wrapped(value, index, data_):
        ret = mapper(fn, value, index, data_)
        if ret is Uninitialized or is_vm_const(ret):
            return ret
        return None

    return _map_vm(data, wrapped)


def map(data: VmAny = Uninitialized, f: VmAny = Uninitialized):
    return _map_impl_wrapped(
        data, "f", f, lambda fn, value, key, data_: Call(fn, *(value, key, data_))
    )


def filter(data: VmAny = Uninitialized, predicate: VmAny = Uninitialized):
    return _map_impl_wrapped(
        data,
        "predicate",
        predicate,
        lambda fn, value, key, data_: (
            value if ToBoolean(Call(fn, *(value, key, data_))) else Uninitialized
        ),
    )


def filter_map(data: VmAny = Uninitialized, f: VmAny = Uninitialized):
    return _map_impl_wrapped(
        data,
        "f",
        f,
        lambda fn, value, key, data_: Call(fn, *(value, key, data_)) or Uninitialized,
    )
