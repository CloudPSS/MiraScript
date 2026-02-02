import { tw } from './_run.ts';

tw('identifier', 'PI', Math.PI);
tw('identifier with whitespace', ' \n SQRT1_2\n\r\n ', Math.SQRT1_2);
tw('global', 'global.E', Math.E);
tw('global dynamic', 'global["E"]', Math.E);

tw('non-existent', 'nonExistent', /Global variable 'nonExistent' is not defined./);
tw('non-existent const', '@nonExistent', /Global variable '@nonExistent' is not defined./);
