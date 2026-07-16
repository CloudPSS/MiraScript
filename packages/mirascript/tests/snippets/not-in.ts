import { t } from './_run.ts';

t(
    'comment in not in',
    `[
    1 /**/ not  in [2, 3, 4],
    1 not /**/ in [2, 3, 4],
    1 not in /**/ [2, 3, 4],
    1 /**/ not /**/ in [2, 3, 4],
    1 not /**/ in /**/ [2, 3, 4],
    1 /**/ not  in /**/ [2, 3, 4],
    1 /**/ not /**/ in /**/ [2, 3, 4],
]`,
    [true, true, true, true, true, true, true],
);
