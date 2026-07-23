import { isVmExtern, isVmModule, type VmValue } from '../../types/index.js';
import { expectString, required, rethrowError, VmLib } from '../helpers.js';
const { parse, stringify } = JSON;

export const to_json = VmLib(
    (data) => {
        required('data', data, null);
        if (typeof data == 'function') return null;
        return stringify(data, (key, value: VmValue) => {
            if (typeof value == 'function') {
                return undefined;
            }
            if (isVmModule(value)) {
                return value.value as JSONValue;
            }
            if (isVmExtern(value)) {
                const extern = value.value;
                if ('toJSON' in extern && typeof extern.toJSON == 'function') {
                    // eslint-disable-next-line @typescript-eslint/no-unsafe-call
                    return extern.toJSON() as JSONValue;
                }
                if (typeof extern == 'function') {
                    return undefined;
                }
                return extern as JSONValue;
            }
            return value as JSONValue;
        });
    },
    {
        summary: '将数据转换为 JSON 字符串',
        params: { data: { type: 'any', description: '要转换为 JSON 的数据' } },
        returns: { type: 'string' },
        examples: ['to_json([1, 2, 3]) // "[1,2,3]"'],
    },
);

export const from_json = VmLib(
    (json, fallback) => {
        const j = expectString('json', json);
        try {
            return parse(j);
        } catch (ex) {
            if (fallback !== undefined) return fallback;
            rethrowError('Invalid JSON', ex, null);
        }
    },
    {
        summary: '将 JSON 字符串转换为数据',
        params: {
            json: { type: 'string', description: '要转换的 JSON 字符串' },
            fallback: { type: 'any', description: '如果转换失败，返回的默认值' },
        },
        returns: { type: 'any' },
        examples: [`from_json('{"a":1}') // (a: 1)`],
    },
);
