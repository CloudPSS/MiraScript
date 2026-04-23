from mirascript.vm.types.const import Uninitialized
from ..._helpers import expect_array, expect_number


def flatten(data=Uninitialized, depth=1):
    expect_array("data", data, data)

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

    d = expect_number("depth", depth)
    return flat(data, d)
