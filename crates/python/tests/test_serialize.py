from __future__ import annotations

import pytest

from mirascript import serialize, display, vm_function, VmModule, VmAny, Uninitialized

EXAMPLES = [
    (Uninitialized, "nil"),
    (None, "nil"),
    (True, "true"),
    (False, "false"),
    (float("nan"), "nan"),
    (float("inf"), "inf"),
    (float("-inf"), "-inf"),
    (0.0, "0"),
    (0, "0"),
    (-0.0, "-0"),
    (1, "1"),
    (-1, "-1"),
    (1.5, "1.5"),
    (-1.5, "-1.5"),
    ("", "''"),
    ("abc", "'abc'"),
    ("a'b'c", "'a\\'b\\'c'"),
    ("a\nb\r\tc", "'a\\nb\\r\\tc'"),
    ("a\0b", "'a\\0b'"),
    ("a\\b", "'a\\\\b'"),
    ("a\u2028b\u2029c", "'a\\u{2028}b\\u{2029}c'"),
    ("a$b", "'a\\$b'"),
    ([1, 2, 3], "[1, 2, 3]"),
    ({"a": 1, "b": 2}, "(a: 1, b: 2)"),
    ([1, [2, [3]]], "[1, [2, [3]]]"),
    ({"a": {"b": {"c": 1}}}, "(a: (b: (c: 1)))"),
    ({}, "()"),
    ([], "[]"),
    ({"0": 1}, "(1,)"),
    ({"0": 1, "1": 2}, "(1, 2)"),
    ({"0": 1, "3": 2}, "(0: 1, 3: 2)"),
    ([float("nan"), float("inf"), -0.0], "[nan, inf, -0]"),
]

SERIALIZE_EXAMPLES = EXAMPLES + [
    (vm_function(lambda x: x + 1), "nil"),
    (VmModule("test_module", {}), "nil"),
]


@pytest.mark.parametrize(("value", "expected"), SERIALIZE_EXAMPLES)
def test_serialize(value: VmAny, expected: str):
    """验证 serialize 的行为。"""
    result = serialize(value)
    assert result == expected, f"serialize({value!r}) = {result}, expected {expected}"


DISPLAY_EXAMPLES = EXAMPLES + [
    ([[[[[]]]]], "[[[[]]]]"),  # depth limit test
    (vm_function(lambda x: x + 1), "<function>"),
    (VmModule("test_module", {}), "<module test_module>"),
    (VmModule("", {}), "<module>"),
]


@pytest.mark.parametrize(("value", "expected"), DISPLAY_EXAMPLES)
def test_display(value: VmAny, expected: str):
    """验证 display 的行为。"""
    result = display(value)
    assert result == expected, f"display({value!r}) = {result}, expected {expected}"
