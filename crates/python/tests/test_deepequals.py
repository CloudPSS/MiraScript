"""deepequals 单元测试。"""

import math

import pytest

from .deepequals import deep_equal

# 测试用例: (a, b, expected)
EXAMPLES = [
    (float("nan"), float("nan"), True),
    (+0.0, -0.0, False),
    (+0.0, +0.0, True),
    (-0.0, -0.0, True),
    ([1, float("nan"), +0.0], [1, float("nan"), -0.0], False),
    ([1, float("nan"), -0.0], [1, float("nan"), -0.0], True),
    ({"a": [1, 2], "b": float("nan")}, {"b": float("nan"), "a": [1, 2]}, True),
    ({"k1": +0.0}, {"k1": -0.0}, False),
    ({1, 2, float("nan")}, {2, float("nan"), 1}, True),
]


@pytest.mark.parametrize(("a", "b", "expected"), EXAMPLES)
def test_deep_equal(a, b, expected):
    """验证 deep_equal 对 NaN、有符号零、嵌套结构的比较行为。"""
    result = deep_equal(a, b)
    assert (
        result == expected
    ), f"deep_equal({a!r}, {b!r}) = {result}, expected {expected}"
