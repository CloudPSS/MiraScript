import { $ToString, $ToNumber } from '../../operations.js';
import { VmLib } from '../_helpers.js';
import { isFinite } from '../../../helpers/utils.js';

export const to_timestamp = VmLib(
    (datetime) => {
        if (datetime == null) {
            return Date.now();
        }
        if (typeof datetime == 'number') {
            return new Date(datetime).getTime();
        }
        const str = $ToString(datetime);
        if (!str) return Number.NaN;
        const num = $ToNumber(str);
        if (isFinite(num)) return num;
        return Date.parse(str);
    },
    {
        summary: '将数据转换为 Unix 毫秒时间戳',
        params: { datetime: '要转换的数据，默认为当前时间' },
        paramsType: { datetime: 'string | number' },
        returnsType: 'number',
        examples: ['to_timestamp("1970-01-01T00:00:00Z") // 0'],
    },
);

export const to_datetime = VmLib(
    (datetime, offset) => {
        const timestamp = to_timestamp(datetime);
        if (!isFinite(timestamp)) return null;
        const o = $ToNumber(offset ?? 0) || 0;
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
        summary: '将数据转换为 Date 记录',
        params: {
            datetime: '要转换的数据，默认为当前时间',
            offset: '时区偏移量（单位：小时），默认为 0',
        },
        paramsType: { datetime: 'string | number', offset: 'number' },
        returnsType: 'Date',
        examples: [
            `
to_datetime(0)
// (
//    year: 1970, month: 1, day: 1,
//    hour: 0, minute: 0, second: 0,
//    millisecond: 0, dayOfWeek: 4, offset: 0
// )
            `.trim(),
        ],
    },
);

export const to_iso8601 = VmLib(
    (datetime) => {
        const timestamp = to_timestamp(datetime);
        if (!isFinite(timestamp)) return null;
        return new Date(timestamp).toISOString();
    },
    {
        summary: '将数据转换为 ISO 8601 格式的字符串',
        params: { datetime: '要转换的数据，默认为当前时间' },
        paramsType: { datetime: 'string | number' },
        returnsType: 'string',
        examples: ['to_iso8601(0) // "1970-01-01T00:00:00.000Z"'],
    },
);
