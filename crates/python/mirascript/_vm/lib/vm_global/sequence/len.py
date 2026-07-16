from ..._helpers import _expect_array


def _len(arr):
    _expect_array("arr", arr, float("nan"))
    return len(arr)
