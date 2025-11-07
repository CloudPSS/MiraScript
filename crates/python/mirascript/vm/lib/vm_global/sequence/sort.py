from ....operations import Call_, ToNumber_, ToString_
from ....types import is_vm_array, is_vm_record, VmValue
from ..._helpers import VmLib, expect_array_or_record,expect_compound

def default_compare(a: VmValue, b: VmValue):
    if a is None and b is None:
        return 0
    if a is None:
        return 1
    if b is None:
        return -1
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return (a > b) - (a < b)
    a_str = ToString_(a)
    b_str = ToString_(b)
    return (a_str > b_str) - (a_str < b_str)
  
def cmp(comparator,recovered):
    def compare(a: VmValue, b: VmValue):
        if comparator is None:
            return default_compare(a, b)
        res = Call_(comparator, a, b)
        if not isinstance(res, (int, float)):
            raise TypeError("比较函数必须返回数字")
        if res != res:  # NaN
            return 0
        return (res > 0) - (res < 0)
    return compare if not recovered else lambda a,b: -compare(a,b)
  
sort = VmLib(
    lambda array, comparator=None, reversed=False: (
        expect_array_or_record('array', array,array),
        expect_const('comparator', comparator,comparator) if comparator is not None else None,
        expect_const('reversed', reversed, reversed) if reversed is not None else None,
        sorted(array, key=lambda x: (0, x) if x is not None else  (1, None), reverse=bool(reversed), cmp=cmp(comparator,comparator is not None))
    )[-1],
    {
        "summary": '对数组中的元素进行排序，并返回排序后的结果',
        "params": { 
          "data": '要排序的数组', 
          "comparator": '用于比较两个元素的函数，返回一个数字，表示它们的相对顺序，默认按升序排列' },
        "paramsType": { 
          "data": 'array', 
          "comparator": 'fn(a: any, b: any) -> number' 
        },
        "returnsType": 'array',
    },
)

sort_by = VmLib(
    lambda array, key, comparator=None, reversed=False: (
        expect_array_or_record('array', array,array),
        expect_callable('key', key,key),
        expect_const('comparator', comparator,comparator) if comparator is not None else None,
        expect_const('reversed', reversed, reversed) if reversed is not None else None,
        sorted(array, key=lambda x: (0, Call_(key, x)) if x is not None else (1, None), reverse=bool(reversed), cmp=cmp(comparator,comparator is not None))
    )[-1],
    {
        "summary": '根据键函数对数组中的元素进行排序，并返回排序后的结果',
        "params": {
            "data": '要排序的数组',
            "key_fn": '用于提取排序键的函数，接受一个元素并返回其排序键',
            "comparator": '用于比较两个排序键的函数，返回一个数字，表示它们的相对顺序，默认按升序排列',
        },
        "paramsType": {
            "data": 'array',
            "key_fn": 'fn(value: any, index: number, arr: type(data)) -> any',
            "comparator": 'fn(a: any, b: any) -> number',
        },
        "returnsType": 'array',
    },
)