from __future__ import annotations
from typing_extensions import TYPE_CHECKING

if TYPE_CHECKING:
    from .types.types import VmAny


class VmError(Exception):
    """VM 预期的错误"""

    def __init__(self, message: str, recovered: VmAny):
        super().__init__(message)
        self.message: str = message
        self.recovered: VmAny = recovered

    @staticmethod
    def from_(prefix: str, error: Exception, recovered: VmAny) -> VmError:
        if prefix:
            if prefix.endswith(":"):
                prefix += " "
            elif not prefix.endswith(": "):
                prefix += ": "
        vmError = VmError(f"{prefix}{str(error)}", recovered)
        vmError.__cause__ = error

        return vmError
