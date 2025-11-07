from .._helpers import required
from ...operations import ToString_
def chars(string):
  required('str', string, None)
  s = ToString_(string)
  return [s[i] for i in range(len(s))]



def starts_with(string, search):
  required('str', string, None)
  required('search', search, None)
  s = ToString_(string)
  p = ToString_(search)
  return s.startswith(p)

def ends_with(string, search): 
  required('str', string, None)
  required('search', search, None)
  s = ToString_(string)
  suf = ToString_(search)
  return s.endswith(suf)

def contains(string, search):
  required('str', string, None)
  required('search', search, None)
  s = ToString_(string)
  sub = ToString_(search)
  return sub in s


def trim_start(string):
  required('str', string, None)
  s = ToString_(string)
  return s.lstrip()

def trim_end(string):
  required('str', string, None)
  s = ToString_(string)
  return s.rstrip()

def trim(string):
  required('str', string, None)
  s = ToString_(string)
  return s.strip()

def replace(string, search, replacement=''):
  required('str', string, None)
  required('search', search, None)
  required('replacement', replacement, None)
  s = ToString_(string)
  old = ToString_(search)
  new = ToString_(replacement)
  return s.replace(old, new)


def split(string, separator=''):
  required('str', string, None)
  s = ToString_(string)
  sep = ToString_(separator)
  return s.split(sep)

def join(string_array, separator=''):
  required('str_array', string_array, None)
  str_list = []
  for item in string_array:
    str_list.append(ToString_(item))
  sep = ToString_(separator)
  return sep.join(str_list)

