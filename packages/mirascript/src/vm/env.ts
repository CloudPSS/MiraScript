import * as operations from './operations.ts';
import * as helpers from './helpers.ts';
import { entries } from '../helpers/utils.ts';

export const keys: string[] = [];
export const values: unknown[] = [];

for (const [key, value] of entries(operations)) {
    keys.push(key);
    values.push(value);
}

for (const [key, value] of entries(helpers)) {
    keys.push(key);
    values.push(value);
}
