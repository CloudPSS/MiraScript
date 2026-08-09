from __future__ import annotations
from abc import ABC, abstractmethod
from typing_extensions import TYPE_CHECKING, Iterable, TypeVar, Generic

if TYPE_CHECKING:
    from . import VmAny, VmTypeName

T = TypeVar("T")


class VmWrapper(ABC, Generic[T]):

    __slots__ = ("__value",)

    def __init__(self, value: T):
        self.__value = value

    @property
    def value(self) -> T:
        return self.__value

    @abstractmethod
    def has(self, key: str) -> bool: ...

    @abstractmethod
    def get(self, key: str) -> VmAny: ...

    @abstractmethod
    def keys(self) -> Iterable[str]: ...

    @abstractmethod
    def same(self, other: VmAny) -> bool: ...

    @property
    @abstractmethod
    def type(self) -> VmTypeName: ...

    @property
    @abstractmethod
    def describe(self) -> str | None: ...

    def __str__(self) -> str:
        if not self.describe:
            return f"<{self.type}>"
        return f"<{self.type} {self.describe}>"
