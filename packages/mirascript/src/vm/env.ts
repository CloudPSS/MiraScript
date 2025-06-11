import * as operations from './operations.js';
import * as helpers from './helpers.js';

export const keys: string[] = [];
export const values: unknown[] = [];

for (const [key, value] of Object.entries(operations)) {
    keys.push(key);
    values.push(value);
}

for (const [key, value] of Object.entries(helpers)) {
    keys.push(key);
    values.push(value);
}
