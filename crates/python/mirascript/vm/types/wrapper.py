from abc import ABC, abstractmethod
from typing_extensions import Sequence, Any, Optional


class VmWrapper(ABC):
    def __init__(self, value):
        self.value = value

    @abstractmethod
    def has(self, key: str) -> bool:
        pass

    @abstractmethod
    def get(self, key: str) -> Any:
        pass

    @abstractmethod
    def keys(self) -> Sequence[str]:
        pass

    @abstractmethod
    def same(self, other: Any) -> bool:
        pass

    @property
    @abstractmethod
    def type(self) -> str:
        pass

    @property
    @abstractmethod
    def describe(self) -> Optional[str]:
        pass

    def __str__(self) -> str:
        if not self.describe:
            return f"<{self.type}>"
        return f"<{self.type} {self.describe}>"
