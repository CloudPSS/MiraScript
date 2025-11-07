from ..._helpers import VmLib, expect_array_or_record,expect_compound

reverse = VmLib(
    lambda data: (
        expect_array_or_record('data', data,data),
        list(reversed(data)) if isinstance(data, list) else dict(reversed(list(data.items())))
    )[-1],
    {
        "summary": '返回数组的反转副本',
        "params": { "arr": '要反转的数组' },
        "paramsType": { "arr": 'array' },
        "returnsType": 'array',
    },
)