from __future__ import annotations

from ....._helpers.convert import to_string
from ....._helpers.constants import Uninitialized
from ....operations import Call
from ....types import VmAny, VmConst
from ..._helpers import _expect_array, _expect_callable


def group_by(data: VmAny = Uninitialized, key_fn: VmAny = Uninitialized):
    data = _expect_array("data", data, None)
    key_fn = _expect_callable("key_fn", key_fn, data)

    result: dict[str, list[VmConst]] = {}

    for i, item in enumerate(data):
        if item is Uninitialized:
            item = None
        key = to_string(Call(key_fn, item, i, data))
        l = result.setdefault(key, [])
        l.append(item)

    return result
