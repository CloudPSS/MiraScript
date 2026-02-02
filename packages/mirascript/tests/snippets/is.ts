import { tw } from './_run.ts';

tw('capture and access', '1 is a && a == 1', true);
tw('access before capture', 'a == 1 && 1 is a', /UninitializedVariable/);
