from ..._helpers import  expect_array

def _len(arr):
    expect_array('arr', arr, float('nan'))
    print('len called with',len(arr))
    return len(arr)

