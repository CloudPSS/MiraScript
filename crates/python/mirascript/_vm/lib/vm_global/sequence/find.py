from ....._helpers.types import is_vm_array, is_vm_callable
from ..._helpers import _expect_array_or_record, _required
from ....operations import Call, ToBoolean, Same
from ....types import Uninitialized


def find(data=Uninitialized, predicate=Uninitialized):
    data = _expect_array_or_record("data", data, None)
    predicate = _required("predicate", predicate, None)
    if is_vm_callable(predicate) == False:

        def p(value, key, data):
            return Same(value, predicate)

    else:

        def p(value, key, data):
            ret = Call(predicate, *(value, key, data))
            return ToBoolean(ret)

    if is_vm_array(data):
        for i, item in enumerate(data):
            value = item if item is not None else None
            if p(value, i, data):
                return {"0": i, "1": value}
        return None
    else:
        for key, v in data.items():
            value = v if v is not None else None
            if p(value, key, data):
                return {"0": key, "1": value}
        return None
