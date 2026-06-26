import test from 'ava';
import * as constants from '@mirascript/constants';
import { isKeyword } from '@mirascript/mirascript/subtle';

test('keywords', (t) => {
    t.false(isKeyword('myVariable'));
    t.false(isKeyword(''));
    t.false(isKeyword(null as never));
    // eslint-disable-next-line unicorn/new-for-builtins
    t.false(isKeyword(new String('nil') as never));
    t.true(isKeyword('nil'));
    t.true(isKeyword('fn'));
    t.true(isKeyword('return'));
});

test('constants', (t) => {
    const testRegExp = (regExp: RegExp, validSamples: string[], invalidSamples: string[]) => {
        t.true(regExp instanceof RegExp, `Expected ${regExp} to be a RegExp`);
        for (const sample of validSamples) {
            t.is(regExp.exec(sample)?.[0], sample, `Expected "${sample}" to match ${regExp}`);
        }
        for (const sample of invalidSamples) {
            t.not(regExp.exec(sample)?.[0], sample, `Expected "${sample}" not to match ${regExp}`);
        }
    };

    testRegExp(
        constants.REG_IDENTIFIER,
        [
            'valid_identifier',
            '变量',
            'πValue',
            '$',
            '$$',
            '$dollarSign',
            '@',
            '@@',
            '@decorator',
            '_',
            '___',
            '_privateVar',
        ],
        ['1invalidStart', 'invalid-char!', 'white space', '', '@$a', '$@a', ' a', 'a '],
    );

    testRegExp(
        constants.REG_ORDINAL,
        ['0', '123', '2147483647', '2147483629'],
        ['-1', '0.', '0.0', '2147483648', '00123'],
    );

    testRegExp(
        constants.REG_WHITESPACE,
        [' ', '\t', '\n', '\r', '\v', '\f'],
        ['\0', 'a', '1', '', ' \t', '\b', '\u0007'],
    );

    testRegExp(
        constants.REG_HEX,
        ['0x1A3F', '0Xabc123', '0x0', '0xDEAD_BEEF', '0x_123'],
        ['123', '0xGHIJ', '0x', '0x1_'],
    );

    testRegExp(
        constants.REG_OCT,
        ['0o1234567', '0O7654321', '0o0', '0o12_34', '0o_123'],
        ['123', '0o89', '0o', '0o1_'],
    );

    testRegExp(
        constants.REG_BIN,
        ['0b101010', '0B1100', '0b0', '0b10_01', '0b_1011'],
        ['123', '0b102', '0b', '0b101_'],
    );

    testRegExp(
        constants.REG_NUMBER,
        ['123', '0.456', '789e10', '3.14E-2', '1_000.00', '6.022e2_3', '6.022e+_23', '1_000_._0__0'],
        ['abc', '123.', '.456', '1e', '1.2.3', /* '12_',  '1._', */ '123e 7'],
    );
});
