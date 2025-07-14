from abc import ABC, abstractmethod
class VmWrapper(ABC):
    def __init__(self):
        pass
    
    @abstractmethod
    def has(self,key):
        pass
    @abstractmethod
    def get(self,key):
        pass
    @abstractmethod
    def keys(self):
        pass
    @abstractmethod
    def same(self,other):
        pass
    
    @property
    @abstractmethod
    def type(self):
        return None
    
    @property
    @abstractmethod
    def describe(self):
        return None
    
    
    ## Convert the object to JSON
    def toJSON(self):
        return None
    
    ## 转为字符串
    def toString(self):
        pass