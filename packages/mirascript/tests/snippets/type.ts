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

tw('type as identifier', 'let type = 1; type(type)', 'number');
tw('type as field name', 'let x = (type: 1); x.type', 1);
tw('type as field name with ?:', 'let x = (type?: 1); x.type', 1);
tw('type as function name', 'fn type() { return 1; }', /InvalidTypeCall/);
tw('type as function name with it args', 'fn type { return it; }', /MissingOpenParenAfterType/);
tw('call type with same name identifier', 'let type = fn {1};type(1)', 'number');
tw('call type identifier', 'let type = fn {1};type!(1)', 1);
