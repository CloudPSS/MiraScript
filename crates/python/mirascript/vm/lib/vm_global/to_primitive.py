from ...operations import ToBoolean_,ToString_,ToNumber_,Format_
from .._helpers import required
import math
from mirascript.vm.types.const import Uninitialized
def to_string(value=Uninitialized):
    required('value', value,'')
    return ToString_(value)

def to_number(value=Uninitialized):
    required('value', value,math.nan)
    return ToNumber_(value)


def to_boolean(value=Uninitialized):
    required('value', value,False)
    return ToBoolean_(value)


def format(value=Uninitialized,fmt=Uninitialized):
    required('value', value,'')
    return Format_(value,fmt)
