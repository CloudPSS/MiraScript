import { tw } from './_run.ts';

tw('keyword', 'nil', null);
tw('keyword', 'nan', Number.NaN);
tw('keyword', 'inf', Number.POSITIVE_INFINITY);
tw('keyword', '+inf', Number.POSITIVE_INFINITY);
tw('keyword', '-inf', Number.NEGATIVE_INFINITY);
tw('keyword', '+ inf', Number.POSITIVE_INFINITY);
tw('keyword', '- inf', Number.NEGATIVE_INFINITY);
tw('keyword', 'true  ', true);
tw('keyword', '  false', false);

tw('number', '1', 1);
tw('number', '1.0', 1);
tw('number', '1.1', 1.1);
tw('number', '1.1e2', 110);
tw('number', '1.1e-2', 0.011);
tw('number', '1.1e+2', 110);
tw('number', '+1', 1);
tw('number', '-1', -1);
tw('number', '0', 0);
tw('number', '-0', -0);
tw('number', '-1e+2', -100);

tw('string', '`hello`', 'hello');
tw('string with escapes', '`hello\\nworld`', 'hello\nworld');
tw('string with unicode escapes', '`\\u{0041}\\u{0042}\\u{0043}`', 'ABC');
tw('empty string', '``', '');
tw('empty string', '""', '');
tw('empty string', "''", '');
tw('empty string', "@''@", '');
tw('empty string', '@@``@@', '');
