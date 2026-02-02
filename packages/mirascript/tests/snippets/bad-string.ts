import { t } from './_run.ts';

t('Unterminated string literal', '`hello', /UnterminatedString/);
t('Unterminated string literal with expr interpolation', '`hello $(name', /UnterminatedInterpolation/);
t('Unterminated string literal with expr format interpolation', '`hello $(name:', /UnterminatedString/);
t('Unterminated string literal with block interpolation', '`hello ${name', /UnterminatedInterpolation/);
t('Empty expr interpolation', '`hello $()!`', /EmptyInterpolation\(1:10\)/);
t('Empty expr interpolation with format', '`hello $(:.2)!`', /EmptyInterpolation\(1:10\)/);
t('Bad expr interpolation with unknown token', '`hello $(~)!`', /EmptyInterpolation\(1:10-11\)/);
t('Bad expr interpolation with unknown tokens', '`hello $(~~)!`', /EmptyInterpolation\(1:10-12\)/);
t('Bad expr interpolation with unknown tokens and format', '`hello $(~~:.1)!`', /EmptyInterpolation\(1:10-12\)/);
