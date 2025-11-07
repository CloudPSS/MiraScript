# -*- coding: utf-8 -*-
# 依赖：VmError, $ToNumber, $Type, is_vm_array, is_vm_extern,  VmExtern, VmFunction, is_vm_record, VmModule, is_vm_primitive, is_vm_const, VM_ARRAY_MAX_LENGTH, Cp
from typing import Optional, TypeVar, TypedDict, Union, cast
from .. import   VmExtern, VmFunction,  VmModule, is_vm_const
from ..operations import ToNumber_, Type_
from ..error import VmError
from ..types.checker import is_vm_array, is_vm_record 
from ..types.const import VM_ARRAY_MAX_LENGTH, VmAny, VmValue
from ..helpers import Cp
from ..types.function import VmFunctionInfo, VmFunctionLike, VmFunctionWrapper
from ..types.const import Uninitialized 
from ..types.checker import is_vm_primitive

def throw_error(message: str, recovered):
	recovered_value = recovered() if callable(recovered) else recovered
	raise VmError(message, recovered_value)

def throw_unexpected_type_error(name  , expected, value, recovered):
	actual = Type_(value)
	if isinstance(name, str):
		throw_error(f"Expected {expected} for parameter '{name}', got {actual}", recovered)
		return
	pos = 'first' if name <= 0 else 'second' if name <= 1 else f'{name+1}th'
	throw_error(f"Expected {expected} at the {pos} position, got {actual}", recovered)

def rethrow_error(prefix: str, error, recovered):
	recovered_value = recovered() if callable(recovered)  else recovered
	raise VmError.from_(prefix, error, recovered_value)

def required(name, value, recovered):
    if value is Uninitialized:
        if isinstance(name, str):
            throw_error(f"Missing required parameter '{name}'", recovered)
            return
        pos = 'first' if name <= 0 else 'second' if name <= 1 else f'{name+1}th'
        throw_error(f"Missing required parameter at the {pos} position", recovered)

def expect_array(name, value, recovered):
	required(name, value, recovered)
	if not is_vm_array(value):
		throw_unexpected_type_error(name, 'array', value, recovered)

def expect_record(name, value, recovered):
	required(name, value, recovered)
	if not is_vm_record(value):
		throw_unexpected_type_error(name, 'record', value, recovered)

def expect_array_or_record(name, value, recovered):
	required(name, value, recovered)
	if not is_vm_array(value) and not is_vm_record(value):
		throw_unexpected_type_error(name, 'array | record', value, recovered)

def expect_compound(name, value, recovered):
	required(name, value, recovered)
	if is_vm_primitive(value) :
		throw_unexpected_type_error(name, 'array | record | module | extern', value, recovered)

def expect_const(name, value, recovered):
	required(name, value, recovered)
	if not is_vm_const(value):
		throw_unexpected_type_error(name, 'nil | number | boolean | string | array | record', value, recovered)

def expect_callable(name, value, recovered):
	required(name, value, recovered)
	callable_ =  callable(value)
	if not callable_:
		throw_unexpected_type_error(name, 'callable', value, recovered)

def get_numbers(args):
	if not args:
		return []
	if len(args) == 1 and is_vm_array(args[0]):
		args = args[0]
	numbers = []
	for arg in args:
		if arg is None:
			continue
		numbers.append(ToNumber_(arg))
	return numbers

def array_len(length):
	if length is None or isinstance(length, float) and (length != length) or length <= 0:
		return 0
	length = int(length)
	if length > VM_ARRAY_MAX_LENGTH:
		throw_error('Array length exceeds maximum', None)
	return length

def map_vm(data, mapper):
	if is_vm_primitive(data):
		return mapper(data, None, data) or None
	if is_vm_array(data):
		result = []
		for i, v in enumerate(data):
			Cp()
			ret = mapper(v if v is not None else None, i, data)
			if ret is None:
				continue
			if is_vm_const(ret):
				result.append(ret)
			else:
				result.append(None)
		return result
	else:
		entries = []
		for key, value in data.items():
			Cp()
			ret = mapper(value if value is not None else None, key, data)
			if ret is None:
				continue
			if is_vm_const(ret):
				entries.append((key, ret))
			else:
				entries.append((key, None))
		return dict(entries)

class VmLibOption(TypedDict, total=False):
	summary: str
	params: dict[str, str]
	paramsType: dict[str, str]
	returns: str
	returnsType: str

class VmLibWrapper(TypedDict, total=False):
	func: VmFunctionLike
	info: VmLibOption
	def __call__(self, *args: Optional[VmValue]) -> Union[VmAny, None]:
			self.func(*args) # type: ignore
        
        
def VmLib(fn, option: VmLibOption) -> VmLibWrapper:
  
		if not callable(fn):
			raise TypeError('Invalid function')

		return VmLibWrapper(func=cast(VmFunctionLike, fn), info=option)
