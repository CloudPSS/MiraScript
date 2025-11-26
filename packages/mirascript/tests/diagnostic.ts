import test from 'ava';
import { getDiagnosticMessage } from '@mirascript/mirascript/subtle';

test('getDiagnosticMessage with invalid code', (t) => {
    const invalidCodes = [-1, 0xffff, 1.5, Number.NaN, Number.POSITIVE_INFINITY, 'x', {}, '1'];
    for (const code of invalidCodes) {
        t.throws(
            () => {
                // @ts-expect-error Testing invalid codes
                getDiagnosticMessage(code);
            },
            { instanceOf: RangeError },
        );
    }
});

test('getDiagnosticMessage with valid code', (t) => {
    // @ts-expect-error Testing invalid codes
    const message = getDiagnosticMessage(0);
    t.is(message, null);
});
