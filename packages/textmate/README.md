# @mirascript/textmate

MiraScript 的 TextMate grammar。该包是 VS Code、Monaco Editor 和其他 Shiki 消费者共享的词法高亮源。

## Shiki

```ts
import { createHighlighter } from 'shiki';
import { mirascript, mirascriptTemplate } from '@mirascript/textmate';

const highlighter = await createHighlighter({
  langs: [mirascript, mirascriptTemplate],
  themes: ['github-dark'],
});
```

包内同时导出以下原始 grammar：

- `@mirascript/textmate/syntaxes/mira.tmLanguage.json`
- `@mirascript/textmate/syntaxes/miratpl.tmLanguage.json`
- `@mirascript/textmate/syntaxes/mira-doc.tmLanguage.json`
