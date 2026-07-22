from __future__ import annotations
import math
import re

from ..._vm.types import VmAny
from ..checker import is_number
from .to_string import to_string


def _format_number(num):
    if num == 0:
        return "0"

    s = to_string(num)

    ps = ""
    absVal = abs(num)
    if absVal >= 1000 or absVal < 0.001:
        ps1 = format(num, f".0e")
        ps2 = format(num, f".5e")
        ps = ps1 if len(ps1) < len(ps2) else ps2

    else:
        ps = format(float(num), f".6")

    return ps if len(ps) < len(s) else s


def to_format(value: VmAny, fmt: str | None = None) -> str:
    fmt = fmt.strip() if fmt is not None else ""

    if is_number(value):
        if not math.isfinite(value):
            return to_string(value)
        r = re.match(r"^\.\d+$", fmt)
        if r:
            ff = float(fmt[1:])
            if ff < 0:
                ff = 0
            elif ff > 100:
                ff = 100
            return format(value, f".{math.trunc(ff)}f")
        else:
            return _format_number(value)
    return to_string(value)
