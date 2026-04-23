from mirascript.vm.operations import ToString_
from ...error import VmError

RED = "\033[91m"
GREEN = "\033[92m"
YELLOW = "\033[93m"
BLUE = "\033[94m"
MAGENTA = "\033[95m"
CYAN = "\033[96m"
WHITE = "\033[97m"
RESET = "\033[0m"


# debug_print=lambda *args: print(CYAN,"MiraScript",*args,RESET),


def debug_print(*args):
    print(CYAN, "MiraScript", *args, RESET)


def panic(*msg):
    # raise RuntimeError(msg)
    if not msg:
        print(RED, "MiraScript", RESET)
    else:
        print(RED, "MiraScript:", *msg, RESET)

    error = f"panic: {ToString_(msg)}" if len(msg) > 0 else "panic"

    raise VmError("MiraScript panic", error)
