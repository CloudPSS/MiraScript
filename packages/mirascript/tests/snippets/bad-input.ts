import { RESERVED_KEYWORDS } from '@mirascript/constants';
import { tw } from './_run.ts';

tw('keyword', 'if', /MissingCloseBrace/);
tw('bad expression', '++', /UnknownExpression/);
tw('bad statement', 'return', /MissingSemicolon/);
tw('global slice', 'global[1..2]', /MisuseOfGlobalKeyword/);

for (const keyword of RESERVED_KEYWORDS) {
    tw(`reserved keyword ${keyword}`, `let ${keyword} = 1;`, /InvalidReservedKeyword/);
}
