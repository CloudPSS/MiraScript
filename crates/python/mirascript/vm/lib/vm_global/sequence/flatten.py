from ..._helpers import VmLib, expect_array

def _flatten(data, depth=1):
    expect_array('data', data, data)
    def flat(arr, d):
        if d < 1:
            return arr
        result = []
        for item in arr:
            if isinstance(item, list):
                result.extend(flat(item, d - 1))
            else:
                result.append(item)
        return result
    return flat(data, int(depth))

flatten = VmLib(
    _flatten,
    {
        "summary": "将数组扁平化",
        "params": {"data": "要扁平化的数组", "depth": "扁平化的深度，默认为 1"},
        "paramsType": {"data": "array", "depth": "number"},
        "returnsType": "array",
    },
)