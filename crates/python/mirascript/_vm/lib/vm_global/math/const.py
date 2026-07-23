from __future__ import annotations
from typing_extensions import Final
import math

__all__ = [
    "pi",
    "PI",
    "e",
    "E",
    "SQRT1_2",
    "SQRT2",
    "LN2",
    "LN10",
    "LOG2E",
    "LOG10E",
]

pi: Final = math.pi
PI: Final = pi
e: Final = math.e
E: Final = e

SQRT1_2: Final = 2**-0.5
SQRT2: Final = 2**0.5
LN2: Final = 0.6931471805599453  # math.log(2)
LN10: Final = 2.302585092994046  # math.log(10)
LOG2E: Final = 1.4426950408889634  # math.log(e, 2)
LOG10E: Final = 0.4342944819032518  # math.log(e, 10)
