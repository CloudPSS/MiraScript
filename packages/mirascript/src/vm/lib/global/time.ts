import { describeParam, expectNumberRange, throwError, throwUnexpectedTypeError, VmLib } from '../helpers.js';
import { isNaN, NotNumber, isFinite } from '../../../helpers/utils.js';
import { toNumber } from '../../../helpers/convert/to-number.js';
import { display } from '../../../helpers/serialize.js';
import type { VmAny } from '../../types/index.js';

const fromNumber = (datetime: number, fallback: boolean): number | null => {
    const n = new Date(datetime).getTime();
    if (isFinite(n)) return n;
    if (fallback) return null;
    throwError(`${describeParam('datetime')} is an invalid timestamp: ${display(datetime)}`, NotNumber);
};

const getTimestamp = (datetime: VmAny, fallback: boolean): number | null => {
    if (datetime == null) {
        return Date.now();
    }
    if (typeof datetime == 'number') {
        return fromNumber(datetime, fallback);
    }
    if (typeof datetime != 'string') {
        if (fallback) return null;
        throwUnexpectedTypeError('datetime', 'number | string', datetime, NotNumber);
    }
    const num = toNumber(datetime, NotNumber);
    if (!isNaN(num)) {
        return fromNumber(num, fallback);
    }
    const parsed = Date.parse(datetime);
    if (isFinite(parsed)) return parsed;
    if (fallback) return null;
    throwError(`${describeParam('datetime')} cannot be parsed as datetime: ${display(datetime)}`, NotNumber);
};

export const to_timestamp = VmLib(
    (datetime, fallback) => {
        const timestamp = getTimestamp(datetime, fallback !== undefined);
        if (timestamp == null) return fallback;
        return timestamp;
    },
    {
        summary: '将数据转换为 Unix 毫秒时间戳',
        params: {
            datetime: '要转换的数据，默认为当前时间',
            fallback: '转换失败时的返回值',
        },
        paramsType: { datetime: 'number | string | nil', fallback: 'any' },
        returnsType: 'number | type(fallback)',
        examples: ['to_timestamp("1970-01-01T00:00:00Z") // 0'],
    },
);

export const to_datetime = VmLib(
    (datetime, offset, fallback) => {
        const timestamp = getTimestamp(datetime, fallback !== undefined);
        if (timestamp == null) return fallback;
        const o = expectNumberRange('offset', offset ?? 0, -24, 24);
        const dateOffset = new Date(timestamp + o * 1000 * 60 * 60);
        return {
            year: dateOffset.getUTCFullYear(),
            month: dateOffset.getUTCMonth() + 1,
            day: dateOffset.getUTCDate(),
            hour: dateOffset.getUTCHours(),
            minute: dateOffset.getUTCMinutes(),
            second: dateOffset.getUTCSeconds(),
            millisecond: dateOffset.getUTCMilliseconds(),
            dayOfWeek: dateOffset.getUTCDay(),
            offset: o,
        };
    },
    {
        summary: '将数据转换为 DateTime 记录',
        params: {
            datetime: '要转换的数据，默认为当前时间',
            offset: '时区偏移量（单位：小时），默认为 0',
            fallback: '转换失败时的返回值',
        },
        paramsType: { datetime: 'number | string | nil', offset: 'number | nil', fallback: 'any' },
        returnsType: 'DateTime | type(fallback)',
        examples: [
            `
to_datetime(0)
// (
//   year: 1970, month: 1, day: 1,
//   hour: 0, minute: 0, second: 0,
//   millisecond: 0,
//   dayOfWeek: 4, offset: 0
// )
            `.trim(),
        ],
    },
);

export const to_iso8601 = VmLib(
    (datetime, fallback) => {
        const timestamp = getTimestamp(datetime, fallback !== undefined);
        if (timestamp == null) return fallback;
        return new Date(timestamp).toISOString();
    },
    {
        summary: '将数据转换为 ISO 8601 格式的字符串',
        params: {
            datetime: '要转换的数据，默认为当前时间',
            fallback: '转换失败时的返回值',
        },
        paramsType: { datetime: 'number | string | nil', fallback: 'any' },
        returnsType: 'string | type(fallback)',
        examples: ['to_iso8601(0) // "1970-01-01T00:00:00.000Z"'],
    },
);
