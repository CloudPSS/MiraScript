from __future__ import annotations
from typing_extensions import NoReturn, TYPE_CHECKING

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
        if prefix and not prefix.endswith(":"):
            prefix += ":"
        vmError = VmError(f"{prefix}{str(error)}", recovered)

        return vmError
