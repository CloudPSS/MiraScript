from ..._helpers import expect_array_or_record, expect_compound, expect_array


def reverse(data):
    expect_array("data", data, None)

    dup = data.copy()
    dup.reverse()
    return dup
