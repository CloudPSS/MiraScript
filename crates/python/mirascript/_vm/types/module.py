from __future__ import annotations
from typing_extensions import Mapping, TYPE_CHECKING

from .wrapper import VmWrapper

if TYPE_CHECKING:
    from . import VmValue


class VmModule(VmWrapper["Mapping[str, VmValue]"]):
    """
    Mirascript 模块包装器
    """

    def __init__(self, name: str, value: "Mapping[str, VmValue]"):
        super().__init__(value)
        self.name = name

    def has(self, key: str) -> bool:
        return key in self.value

    def get(self, key: str) -> VmValue:
        return self.value.get(key, None)

    def keys(self) -> list[str]:
        return list(self.value.keys())

    def same(self, other: VmValue) -> bool:
        return self is other

    @property
    def type(self) -> str:
        return "module"

    @property
    def describe(self) -> str:
        return self.name
