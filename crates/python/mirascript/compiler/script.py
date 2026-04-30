from typing_extensions import Protocol
from ast import Module

from ..helpers.constants import kVmScript
from ..vm.types.context import VmContext
from ..vm.types.types import VmValue


class VmScriptLike(Protocol):
    """A protocol representing a callable object that can be executed with an optional global context."""

    def __call__(self, global_ctx: "VmContext | None" = None) -> VmValue: ...


class VmScript(VmScriptLike):
    """A class representing a compiled MiraScript script, which is a callable object with additional metadata."""

    ast: Module
    filename: "str | None"
    source: "str | None"


def wrap_vm_script(
    func: "VmScriptLike | Exception",
    *,
    filename: "str | None" = None,
    source: "str | None" = None,
    ast: "Module | None" = None
) -> "VmScript":
    if isinstance(func, Exception):
        err = func

        def error_func(global_ctx: "VmContext | None" = None, *args, **kwargs):
            raise err

        func = error_func

    setattr(func, kVmScript, True)
    setattr(func, "ast", ast)
    setattr(func, "filename", filename)
    setattr(func, "source", source)
    return func  # type: ignore
