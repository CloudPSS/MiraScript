from __future__ import annotations
from typing_extensions import Protocol, cast, TYPE_CHECKING
from ast import Module

from .._helpers.constants import kVmScript

if TYPE_CHECKING:
    from .._compiler import InputMode
    from .._vm.types import VmValue, VmContextLike


class VmScriptLike(Protocol):
    """A protocol representing a callable object that can be executed with an optional global context."""

    def __call__(self, global_ctx: VmContextLike | None = None) -> VmValue: ...


class VmScript(VmScriptLike):
    """A class representing a compiled MiraScript script, which is a callable object with additional metadata."""

    ast: Module | None
    filename: str
    source: str
    input_mode: InputMode


def wrap_vm_script(
    func: VmScriptLike | Exception,
    *,
    filename: str,
    input_mode: InputMode,
    source: str,
    ast: Module | None,
) -> VmScript:
    if isinstance(func, Exception):

        def error_func(global_ctx: VmContextLike | None = None, *, err=func):
            raise err

        func = error_func

    setattr(func, kVmScript, True)
    setattr(func, "ast", ast)
    setattr(func, "filename", filename)
    setattr(func, "source", source)
    setattr(func, "input_mode", input_mode)
    return cast(VmScript, func)
