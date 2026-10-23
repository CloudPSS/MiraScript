"""Compatibility cases where Python's complex arithmetic has different semantics."""

from __future__ import annotations

import math
from importlib import import_module

import pytest

c = import_module("mirascript._vm.lib.vm_global.mod.complex")


def record(real: float, imag: float) -> dict[str, float]:
    return {"0": real, "1": imag}


@pytest.mark.parametrize(
    ("name", "args", "expected"),
    [
        ("polar", (0.0, math.inf), (math.nan, math.nan)),
        ("polar", (0.0, math.nan), (math.nan, math.nan)),
        ("polar", (math.inf, math.nan), (math.nan, math.nan)),
        ("polar", (math.nan, 0.0), (math.nan, math.nan)),
        (
            "multiply",
            (record(math.inf, math.inf), record(1.0, 0.0)),
            (math.nan, math.nan),
        ),
        ("divide", (record(-2.0, -2.0), record(-2.0, -2.0)), (1.0, 0.0)),
        ("divide", (record(-2.0, -2.0), record(-2.0, 2.0)), (0.0, 1.0)),
        ("pow", (record(-1e308, -1e308), -2.0), (-0.0, -0.0)),
        ("atan", (record(math.inf, 1e308),), (math.pi / 2, math.nan)),
        ("atanh", (record(1e308, math.inf),), (math.nan, math.pi / 2)),
        ("asin", (record(math.inf, math.nan),), (math.nan, math.inf)),
        ("acos", (record(math.inf, math.nan),), (math.nan, -math.inf)),
    ],
)
def test_complex_special_values(name: str, args: tuple, expected: tuple[float, float]):
    result = getattr(c, name)(*args)
    assert result.keys() == {"0", "1"}
    for key, want in zip(("0", "1"), expected):
        got = result[key]
        if math.isnan(want):
            assert math.isnan(got)
        else:
            assert got == want
            if want == 0:
                assert math.copysign(1.0, got) == math.copysign(1.0, want)
