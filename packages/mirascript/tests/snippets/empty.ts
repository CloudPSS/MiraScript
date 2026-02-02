import { tw } from './_run.ts';

tw('empty', '', null);
tw('whitespace', ' ', null);
tw('whitespaces', ' \n', null);
tw('comments', '// this is a comment\n/* multi-line\n comment */', null);
tw('whitespaces and comments', ' \n // comment \n /* multi-line \n comment */ ', null);
tw('semicolons', ';;;', null);
tw('whitespaces, comments and semicolons', ' \n // comment \n ;;; /* multi-line \n comment */ ;;; ', null);
tw('empty block', '{}', null);
tw('block with whitespaces', '{   \n  }', null);
tw('block with comments', '{ // comment \n /* multi-line \n comment */ }', null);
tw('block with semicolons', '{;;;}', null);
tw('block with whitespaces, comments and semicolons', '{ \n // comment \n ;;; /* multi-line \n comment */ ;;; }', null);
tw('nested empty blocks', '{{{}}}', null);
tw(
    'nested empty blocks with whitespaces, comments and semicolons',
    '{ \n // comment \n ;;; { /* multi-line \n comment */ ;;; {   \n  } ;;; } ;;; }',
    null,
);
