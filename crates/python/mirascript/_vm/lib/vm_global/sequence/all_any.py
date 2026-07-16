from mirascript._helpers.types import is_vm_array
from mirascript._vm.types.types import Uninitialized
from ..._helpers import _expect_array_or_record, _expect_callable
from ....operations import Call, ToBoolean

__all__ = ["all", "any"]


def all(data=Uninitialized, fn=Uninitialized):
    _expect_array_or_record("data", data, None)
    _expect_callable("fn", fn, data)

    if is_vm_array(data):
        for i, item in enumerate(data):
            ret = Call(fn, *(item, i, data))
            if not ToBoolean(ret):
                return False
        return True
    else:
        for key, item in data.items():
            ret = Call(fn, *(item, key, data))
            if not ToBoolean(ret):
                return False
        return True


def any(data=Uninitialized, fn=Uninitialized):
    _expect_array_or_record("data", data, None)
    _expect_callable("fn", fn, data)

    if is_vm_array(data):
        for i, item in enumerate(data):
            ret = Call(fn, *(item, i, data))
            if ToBoolean(ret):
                return True
        return False
    else:
        for key, item in data.items():
            ret = Call(fn, *(item, key, data))
            if ToBoolean(ret):
                return True
        return False
