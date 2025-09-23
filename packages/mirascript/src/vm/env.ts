import * as operations from './operations.js';
import * as helpers from './helpers.js';
import { entries } from '../helpers/utils.js';

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
