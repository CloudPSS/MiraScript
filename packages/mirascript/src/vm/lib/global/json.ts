import { isVmExtern, isVmModule } from '../../types/index.js';
import { required, rethrowError, VmLib } from '../_helpers.js';
const { parse, stringify } = JSON;

export const to_json = VmLib(
    (data) => {
        let value;
        if (isVmExtern(data) || isVmModule(data)) {
            value = data.value;
        } else {
            value = data;
        }
        try {
            return stringify(value);
        } catch (ex) {
            rethrowError('Failed to convert Extern to JSON', ex, '{}');
        }
    },
    {
        summary: '将数据转换为 JSON 字符串',
        params: { data: '要转换为 JSON 的数据' },
        paramsType: { data: 'any' },
        returnsType: 'string',
        examples: ['to_json([1, 2, 3]) // "[1,2,3]"'],
    },
);

export const from_json = VmLib(
    (json, fallback) => {
        required('json', json, null);
        if (typeof json != 'string') return json;
        try {
            return parse(json);
        } catch (ex) {
            if (fallback != null) return fallback;
            rethrowError('Invalid JSON', ex, null);
        }
    },
    {
        summary: '将 JSON 字符串转换为数据',
        params: { json: '要转换的 JSON 字符串', fallback: '如果转换失败，返回的默认值' },
        paramsType: { json: 'string', fallback: 'any' },
        returnsType: 'any',
        examples: [`from_json('{"a":1}') // (a: 1)`],
    },
);
