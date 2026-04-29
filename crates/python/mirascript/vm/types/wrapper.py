from abc import ABC, abstractmethod


class VmWrapper(ABC):
    def __init__(self, value):
        self.value = value

    @abstractmethod
    def has(self, key):
        pass

    @abstractmethod
    def get(self, key):
        pass

    @abstractmethod
    def keys(self):
        pass

    @abstractmethod
    def same(self, other):
        pass

    @property
    @abstractmethod
    def type(self):
        return None

    @property
    @abstractmethod
    def describe(self):
        return None

    def __str__(self) -> str:
        if not self.describe:
            return f"<{self.type}>"
        return f"<{self.type} {self.describe}>"
