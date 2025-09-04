import { $ToString, $ToNumber } from '../../operations.js';
import { VmLib } from '../_helpers.js';

export const to_timestamp = VmLib(
    (data) => {
        data ??= Date.now();
        if (typeof data == 'number') return data;
        const str = $ToString(data);
        if (!str) return Number.NaN;
        const num = $ToNumber(str);
        if (Number.isFinite(num)) return num;
        return Date.parse(str);
    },
    {
        summary: '将数据转换为 Unix 毫秒时间戳',
        params: { data: '要转换的数据，默认为当前时间' },
        paramsType: { data: 'string | number' },
        returnsType: 'number',
    },
);

export const to_datetime = VmLib(
    (data, offset) => {
        const timestamp = to_timestamp(data);
        if (!Number.isFinite(timestamp)) return null;
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
        summary: '将数据转换为 Date 对象',
        params: {
            data: '要转换的数据，默认为当前时间',
            offset: '时区偏移量（单位：小时），默认为 0',
        },
        paramsType: { data: 'string | number', offset: 'number' },
        returnsType: 'Date',
    },
);

export const to_iso8601 = VmLib(
    (data) => {
        const timestamp = to_timestamp(data);
        if (!Number.isFinite(timestamp)) return null;
        return new Date(timestamp).toISOString();
    },
    {
        summary: '将数据转换为 ISO 8601 格式的字符串',
        params: { data: '要转换的数据，默认为当前时间' },
        paramsType: { data: 'string | number' },
        returnsType: 'string',
    },
);
