from __future__ import annotations

from ....types import Uninitialized, VmAny
from ..._helpers import _expect_callable, _expect_const, _iterate, _required
from ....operations import Call


def fold(
    data: VmAny = Uninitialized,
    initial: VmAny = Uninitialized,
    f: VmAny = Uninitialized,
):
    data = _expect_const("data", data, None)
    initial = _required("initial", initial, None)
    f = _expect_callable("f", f, data)

    acc = initial

    def wrapped(value, index, data):
        nonlocal acc
        acc = Call(f, acc, value, index, data)
        return Uninitialized

    _iterate(data, wrapped)
    return acc
