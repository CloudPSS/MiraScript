# -*- coding: utf-8 -*-

K_VM_SCRIPT = '__mirascript_vm_script__'

class VmScript:
	"""
	Mirascript 脚本对象，包装可调用对象及元数据
	"""
	def __init__(self, func, source: str):
		self.func = func
		self.source = source
		setattr(self, K_VM_SCRIPT, True)
	def __call__(self, global_ctx=None):
		return self.func(global_ctx) if global_ctx is not None else self.func()

def is_vm_script(value) -> bool:
	return callable(value) and getattr(value, K_VM_SCRIPT, False) is True
