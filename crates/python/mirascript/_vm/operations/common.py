from __future__ import annotations
from ..error import VmError
from ..types import Uninitialized, VmAny


def AssertInit(val: VmAny):
    if val is Uninitialized:
        raise VmError("Uninitialized value`", None)
