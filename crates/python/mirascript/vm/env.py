from typing import Callable

from . import operations, helpers

vm_globals: "dict[str, Callable]" = {}

for k in dir(operations):
    if not k.startswith("_"):
        vm_globals[k] = getattr(operations, k)
for k in dir(helpers):
    if not k.startswith("_"):
        vm_globals[k] = getattr(helpers, k)
