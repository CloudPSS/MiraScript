from mirascript.vm.operations import ToString_
from ...error import VmError


DEBUG = "\033[44;37m"
PANIC = "\033[41;37m"
RESET = "\033[0m"


# debug_print=lambda *args: print(CYAN,"MiraScript",*args,RESET),


def debug_print(*args):
    print(f"{DEBUG} MiraScript {RESET}", *args, )


def panic(*msg):
    # raise RuntimeError(msg)
    if not msg:
        print(f"{PANIC} MiraScript {RESET}")
    else:
        print(f"{PANIC} MiraScript {RESET}", *msg)

    error = f"panic: {ToString_(msg)}" if len(msg) > 0 else "panic"

    raise VmError("MiraScript panic", error)
