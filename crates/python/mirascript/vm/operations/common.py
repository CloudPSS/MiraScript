from ..error import VmError
from ..types.types import Uninitialized, VmAny


def AssertInit(val: VmAny):
    if val is Uninitialized:
        raise VmError("Uninitialized value`", None)
