import math
import re

from mirascript.vm.types.const import Uninitialized
from .to_string import numberToString_, toString


def formatNumber(num):
    if not math.isfinite(num):
        return toString(num)

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


def toFormat(val, fmt=None):
    f = ""
    if fmt is not None and fmt is not Uninitialized:
        f = fmt.strip()

    if isinstance(val, bool):
        return toString(val)

    if isinstance(val, (int, float)):
        r = re.match(r"^\.\d+$", f)
        if r:
            ff = float(f[1:])
            if math.isinf(ff):
                digits = 100
            else:
                digits = math.trunc(ff)
                if not (digits <= 100):
                    digits = 100
            return f"{val:.{digits}f}"
        else:
            return formatNumber(val)
    return toString(val)
