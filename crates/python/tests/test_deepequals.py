"""deepequals 单元测试。"""

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
    (1, 1.0, True),
    (1, True, False),
    (0, 0.0, True),
    (0, -0.0, False),
    (0, False, False),
    ("abc", "abc", True),
    ("abc", "def", False),
    (b"bytes", b"bytes", True),
    (b"bytes", b"other", False),
    (bytearray(b"abc"), bytearray(b"abc"), True),
    (bytearray(b"abc"), bytearray(b"def"), False),
    (bytearray(b"abc"), b"abc", True),
    ("abc", b"abc", False),
    ("abc", ["a", "b", "c"], False),
]


@pytest.mark.parametrize(("a", "b", "expected"), EXAMPLES)
def test_deep_equal(a, b, expected):
    """验证 deep_equal 对 NaN、有符号零、嵌套结构的比较行为。"""
    result = deep_equal(a, b)
    assert (
        result == expected
    ), f"deep_equal({a!r}, {b!r}) = {result}, expected {expected}"
