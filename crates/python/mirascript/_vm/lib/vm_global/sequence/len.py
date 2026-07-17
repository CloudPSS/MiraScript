import builtins
from ..._helpers import _expect_array


def len(arr):
    arr = _expect_array("arr", arr, float("nan"))
    return builtins.len(arr)
