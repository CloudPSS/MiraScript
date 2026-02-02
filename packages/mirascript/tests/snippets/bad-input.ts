import { tw } from './_run.ts';

tw('keyword', 'if', /MissingCloseBrace/);
tw('bad expression', '++', /UnknownExpression/);
tw('bad statement', 'return', /MissingSemicolon/);
