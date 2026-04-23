# -*- coding: utf-8 -*-
from abc import ABC, abstractmethod
from .const import VmAny

class VmWrapper(ABC):
	"""
	Mirascript 特殊值的包装器基类
	"""
	def __init__(self, value):
		self.value = value

	@abstractmethod
	def has(self, key: str) -> bool:
		pass

	@abstractmethod
	def get(self, key: str) -> VmAny:
		pass

	@abstractmethod
	def keys(self) -> list[str]:
		pass

	@abstractmethod
	def same(self, other) -> bool:
		pass

	@property
	@abstractmethod
	def type(self)->str:
		pass

	@property
	@abstractmethod
	def describe(self) -> str:
		pass

	def toJSON(self):
		return None

	def __str__(self):
		return f'<{self.type} {self.describe}>'


class VmModule(VmWrapper):
	"""
	Mirascript 模块包装器
	"""
	def __init__(self, name: str, value: dict):
		super().__init__(value)
		self.name = name

	def has(self, key: str) -> bool:
		return key in self.value

	def get(self, key: str) -> VmAny:
		if not self.has(key):
			return None
		return self.value.get(key, None)

	def keys(self) -> list[str]:
		return list(self.value.keys())

	def same(self, other) -> bool:
		return self is other

	@property
	def type(self):
		return 'module'

	@property
	def describe(self) -> str:
		return self.name


class VmExtern(VmWrapper):
	def __init__(self, value, caller=None):
		super().__init__(value)
		self.caller = caller

	def _access(self, key: str, read: bool) -> bool:
		if key.startswith('_'):
			return False
		if callable(self.value) and key in ('prototype', 'arguments', 'caller'):
			return False
		if hasattr(self.value, key):
			return True
		if not read:
			return True
		if not hasattr(self.value, key):
			return False
		if key == 'constructor':
			return False
		# 跳过内建属性
		return True

	def has(self, key: str) -> bool:
		return self._access(key, True)

	def get(self, key: str) -> Any:
		if not self.has(key):
			return None
		prop = getattr(self.value, key, None)
		return wrap_to_vm_value(prop, self)

	def set(self, key: str, value):
		if not self._access(key, False):
			return False
		prop = unwrap_from_vm_value(value)
		setattr(self.value, key, prop)
		return True

	def call(self, args):
		if not callable(self.value):
			raise Exception('Not a callable extern')
		caller = self.caller.value if self.caller else None
		unwrapped_args = [unwrap_from_vm_value(a) for a in args]
		# Python 没有 Reflect.apply，直接调用
		ret = self.value(*unwrapped_args)
		return wrap_to_vm_value(ret, self)

	@property
	def callable(self) -> bool:
		return callable(self.value)

	def keys(self) -> list[str]:
		keys = []
		for key in dir(self.value):
			if self.has(key):
				keys.append(key)
		return keys

	def same(self, other) -> bool:
		return isinstance(other, VmExtern) and self.value is other.value and self.caller is other.caller

	@property
	def type(self):
		return 'extern'

	@property
	def describe(self) -> str:
		return type(self.value).__name__

