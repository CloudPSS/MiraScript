from ....types import Uninitialized, VmAny
from ..._helpers import _expect_array, _expect_number


def flatten(data: VmAny = Uninitialized, depth: VmAny = 1):
    _expect_array("data", data, data)

    def flat(arr, d):
        if d < 1:
            return arr
        result = []
        for item in arr:
            if isinstance(item, list):
                result.extend(flat(item, d - 1))
            else:
                result.append(item)
        return result

    d = _expect_number("depth", depth)
    return flat(data, d)
