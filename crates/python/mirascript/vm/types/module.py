# -*- coding: utf-8 -*-
from typing import Any
from .wrapper import VmWrapper

class VmModule(VmWrapper):
	"""
	Mirascript 模块包装器
	"""
	def __init__(self, name: str, value: dict):
		super().__init__(value)
		self.name = name

	def has(self, key):
		return key in self.value

	def get(self, key: str) -> Any:
		if not self.has(key):
			return None
		return self.value.get(key, None)

	def keys(self):
		return list(self.value.keys())

	def same(self, other) -> bool:
		return self is other

	@property
	def type(self):
		return 'module'

	@property
	def describe(self) -> str:
		return self.name
