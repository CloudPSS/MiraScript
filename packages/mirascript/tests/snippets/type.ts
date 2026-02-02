import { tw } from './_run.ts';

tw('identifier', 'type(PI)', 'number');
tw('number', 'type(1)', 'number');
tw('string', 'type("hello")', 'string');
tw('boolean true', 'type(true)', 'boolean');
tw('boolean false', 'type(false)', 'boolean');
tw('nil', 'type(nil)', 'nil');
tw('function', 'type(fn{})', 'function');
tw('array', 'type([])', 'array');
tw('record', 'type(())', 'record');

tw('with trailing comma', 'type(1,)', 'number');
