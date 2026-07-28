from __future__ import annotations

import math

import pytest

from mirascript import to_boolean, to_string, to_number, to_format, VmError, VmValue


def test_to_boolean():
    assert to_boolean(True) is True
    assert to_boolean(False) is False
    assert to_boolean(1, fallback=False) is False
    with pytest.raises(Exception):
        to_boolean(1)


def test_to_string():
    assert to_string("abc") == "abc"
    assert to_string(123) == "123"
    assert to_string(-0.0) == "0"
    assert to_string(float("nan")) == "nan"
    assert to_string(float("inf")) == "inf"
    assert to_string(-float("inf")) == "-inf"
    assert to_string({"a": -0.0, "b": 1}) == "a: 0, b: 1"
    assert to_string([float("nan"), -0.0, 0.0, 0, 1]) == "nan, 0, 0, 0, 1"

    class CantString:
        def __str__(self):
            raise Exception("Cannot convert to string")

    value: VmValue = CantString()  # pyright: ignore[reportAssignmentType]

    assert to_string(value, fallback="fallback") == "fallback"
    with pytest.raises(VmError):
        to_string(value)


def test_to_number():
    assert to_number(123) == 123
    assert to_number(-0.0) == -0.0
    assert math.isnan(to_number(float("nan")))
    assert to_number(float("inf")) == float("inf")
    assert to_number(-float("inf")) == -float("inf")
    assert to_number("123") == 123
    assert to_number("-0") == 0 and math.copysign(1, to_number("-0")) == -1
    assert to_number("0x1A") == 26
    assert to_number("0b101") == 5
    assert to_number("0o17") == 15
    assert to_number("1.23e4") == 12300.0
    assert to_number("  42  ") == 42
    assert math.isnan(to_number("nan"))
    assert to_number("inf") == float("inf")
    assert to_number("-inf") == -float("inf")

    value: VmValue = [1]

    assert to_number(value, fallback=12) == 12
    with pytest.raises(VmError):
        to_number(value)


def test_to_format():
    assert to_format("abc") == "abc"
    assert to_format(123) == "123"
    assert to_format(-0.0) == "0"
    assert to_format(float("nan")) == "nan"
    assert to_format(float("inf")) == "inf"
    assert to_format(-float("inf")) == "-inf"
    assert to_format({"a": -0.0, "b": 1}) == "a: 0, b: 1"
    assert to_format([float("nan"), -0.0, 0.0, 0, 1]) == "nan, 0, 0, 0, 1"
    assert to_format(math.pi) == "3.14159"
    assert to_format(math.pi, ".0") == "3"
    assert to_format(math.pi, ".-1") == "3.14159"
