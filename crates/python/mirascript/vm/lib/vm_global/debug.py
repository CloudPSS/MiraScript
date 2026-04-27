import sys
from ...operations import ToString_
from ...error import VmError


DEBUG = "\033[44;37m"
PANIC = "\033[41;37m"
RESET = "\033[0m"

DEBUG_PREFIX = f"{DEBUG} MiraScript {RESET}" if sys.stdout.isatty() else "[MiraScript]"
PANIC_PREFIX = f"{PANIC} MiraScript {RESET}" if sys.stdout.isatty() else "[MiraScript]"


def debug_print(*args):
    print(DEBUG_PREFIX, *args)


def panic(*msg):
    if not msg:
        print(PANIC_PREFIX)
    else:
        print(PANIC_PREFIX, *msg)

    error = f"panic: {ToString_(msg)}" if len(msg) > 0 else "panic"

    raise VmError("MiraScript panic", error)
