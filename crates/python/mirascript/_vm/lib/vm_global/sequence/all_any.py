from ....._helpers.types import is_vm_array
from ....types import Uninitialized, VmAny
from ..._helpers import _expect_array_or_record, _expect_callable
from ....operations import Call, ToBoolean

__all__ = ["all", "any"]


def _build(every: bool):
    def inner(data: VmAny = Uninitialized, fn: VmAny = Uninitialized):
        data = _expect_array_or_record("data", data, None)
        fn = _expect_callable("fn", fn, data)

        if is_vm_array(data):
            for i, item in enumerate(data):
                ret = Call(fn, item, i, data)
                if ToBoolean(ret) is not every:
                    return not every
            return every
        else:
            for key, item in data.items():
                ret = Call(fn, item, key, data)
                if ToBoolean(ret) is not every:
                    return not every
            return every

    return inner


all = _build(True)
any = _build(False)
