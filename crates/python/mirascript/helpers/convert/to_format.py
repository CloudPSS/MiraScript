import math
import re

from .to_string import toString


def formatNumber(num):
    if num == 0:
        return "0"

    s = toString(num)

    ps = ""
    absVal = abs(num)
    if absVal >= 1000 or absVal < 0.001:
        ps1 = format(num, f".0e")
        ps2 = format(num, f".5e")
        ps = ps1 if len(ps1) < len(ps2) else ps2

    else:
        ps = format(float(num), f".6")

    return ps if len(ps) < len(s) else s


def toFormat(val, fmt: "str | None" = None) -> str:
    fmt = fmt.strip() if fmt is not None else ""

    if isinstance(val, bool):
        return toString(val)

    if isinstance(val, (int, float)):
        if not math.isfinite(val):
            return toString(val)
        r = re.match(r"^\.\d+$", fmt)
        if r:
            ff = float(fmt[1:])
            if ff < 0:
                ff = 0
            elif ff > 100:
                ff = 100
            return f"{val:.{math.trunc(ff)}f}"
        else:
            return formatNumber(val)
    return toString(val)
