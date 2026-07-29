from __future__ import annotations
from typing_extensions import Callable

from ....._helpers.convert import to_boolean
from ....._helpers.types import is_vm_const
from ....types import Uninitialized, VmAny, VmFunction
from ..._helpers import _expect_callable, _expect_const, _iterate
from ....operations import Call


def _map_impl_wrapped(
    data: VmAny,
    fn_name: str,
    fn: VmAny,
    mapper: Callable[[VmFunction, VmAny, VmAny, VmAny], VmAny],
) -> VmAny:
    data = _expect_const("data", data, None)
    fn = _expect_callable(fn_name, fn, data)

    def wrapped(value, index, data):
        ret = mapper(fn, value, index, data)
        if ret is Uninitialized or is_vm_const(ret):
            return ret
        return None

    return _iterate(data, wrapped)


def map(data: VmAny = Uninitialized, f: VmAny = Uninitialized):
    def wrapper(fn, value, key, data):
        return Call(fn, value, key, data)

    return _map_impl_wrapped(data, "f", f, wrapper)


def filter(data: VmAny = Uninitialized, predicate: VmAny = Uninitialized):
    def wrapper(fn, value, key, data):
        result = Call(fn, value, key, data)
        return value if to_boolean(result) else Uninitialized

    return _map_impl_wrapped(data, "predicate", predicate, wrapper)


def filter_map(data: VmAny = Uninitialized, f: VmAny = Uninitialized):
    def wrapper(fn, value, key, data):
        result = Call(fn, value, key, data)
        return result if result is not None else Uninitialized

    return _map_impl_wrapped(data, "f", f, wrapper)
