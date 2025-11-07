import math
import random
from ...operations import ToNumber_


def atan2(y,x):
    return math.atan2(ToNumber_(y),ToNumber_(x))
def pow(x,y):
    return math.pow(ToNumber_(x),ToNumber_(y))
def random_():
  return random.random()