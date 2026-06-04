import sys
from ...operations import ToString
from ...error import VmError

_DEBUG = "\033[44;37m"
_PANIC = "\033[41;37m"
_RESET = "\033[0m"

_DEBUG_PREFIX = (
    f"{_DEBUG} MiraScript {_RESET}" if sys.stdout.isatty() else "[MiraScript]"
)
_PANIC_PREFIX = (
    f"{_PANIC} MiraScript {_RESET}" if sys.stdout.isatty() else "[MiraScript]"
)


def debug_print(*args):
    print(_DEBUG_PREFIX, *args)


def panic(msg=None):
    if not msg:
        print(_PANIC_PREFIX)
    else:
        print(_PANIC_PREFIX, msg)

    error = f"panic: {ToString(msg)}" if msg is not None else "panic"

    raise VmError("MiraScript panic", error)
