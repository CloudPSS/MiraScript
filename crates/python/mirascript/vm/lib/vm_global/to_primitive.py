from ...operations import ToBoolean_,ToString_,ToNumber_,Format_
from .._helpers import required
import math
def to_string(value):
    required('value', value,'')
    return ToString_(value)

def to_number(value):
    required('value', value,math.nan)
    return ToNumber_(value)


def to_boolean(value):
    required('value', value,False)
    return ToBoolean_(value)


def format(value,fmt):
    required('value', value,'')
    return Format_(value,fmt)
