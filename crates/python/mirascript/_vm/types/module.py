from __future__ import annotations
from typing_extensions import Mapping, Iterable, TYPE_CHECKING, TypeAlias

from .wrapper import VmWrapper

if TYPE_CHECKING:
    from . import VmValue, VmAny, VmTypeName

    ModuleDict: TypeAlias = Mapping[str, VmValue]

else:
    ModuleDict = Mapping


class VmModule(VmWrapper[ModuleDict]):
    """
    Mirascript 模块包装器
    """

    __slots__ = ("name",)

    def __init__(self, name: str, value: ModuleDict):
        super().__init__(value)
        self.name = name

    def has(self, key: str) -> bool:
        return key in self.value

    def get(self, key: str) -> VmValue:
        return self.value.get(key, None)

    def keys(self) -> Iterable[str]:
        return self.value.keys()

    def same(self, other: VmAny) -> bool:
        return self is other

    @property
    def type(self) -> VmTypeName:
        return "module"

    @property
    def describe(self) -> str:
        return self.name
