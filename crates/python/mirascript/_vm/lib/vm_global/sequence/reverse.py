from ..._helpers import _expect_array


def reverse(data):
    _expect_array("data", data, None)

    dup = data.copy()
    dup.reverse()
    return dup
