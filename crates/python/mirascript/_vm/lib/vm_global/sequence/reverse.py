from ..._helpers import _expect_array_or_record, _expect_compound, _expect_array


def reverse(data):
    _expect_array("data", data, None)

    dup = data.copy()
    dup.reverse()
    return dup
