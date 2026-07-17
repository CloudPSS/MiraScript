from mirascript._helpers.types import is_vm_array
from mirascript._vm.types.types import Uninitialized, VmAny
from ..._helpers import _expect_array_or_record, _expect_compound


def keys(data: VmAny = Uninitialized):
    data = _expect_compound("data", data, [])

    if is_vm_array(data):
        return list(range(len(data)))
    else:
        return list(data.keys())


def values(data=Uninitialized):
    data = _expect_array_or_record("data", data, [])

    if is_vm_array(data):
        return list(data)
    else:
        return list(data.values())


def entries(data=Uninitialized):
    data = _expect_array_or_record("data", data, [])
    if is_vm_array(data):
        # return list(enumerate(data)) # type: ignore

        ret = []
        for i, v in enumerate(data):
            ret.append({"0": i, "1": v if v is not None else None})
        return ret

    ret = []
    for key, v in data.items():
        ret.append({"0": key, "1": v if v is not None else None})

    return ret
