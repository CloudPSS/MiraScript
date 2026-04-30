from typing_extensions import Mapping, TYPE_CHECKING

from .wrapper import VmWrapper

if TYPE_CHECKING:
    from .types import VmValue


class VmModule(VmWrapper["Mapping[str, VmValue]"]):
    """
    Mirascript 模块包装器
    """

    def __init__(self, name: str, value: "Mapping[str, VmValue]"):
        super().__init__(value)
        self.name = name

    def has(self, key):
        return key in self.value

    def get(self, key) -> "VmValue":
        return self.value.get(key, None)

    def keys(self):
        return list(self.value.keys())

    def same(self, other) -> bool:
        return self is other

    @property
    def type(self):
        return "module"

    @property
    def describe(self) -> str:
        return self.name
