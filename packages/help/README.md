# @mirascript/help

`@mirascript/help` 用于将仓库中的参考文档编译为可直接消费的帮助数据，当前主要生成关键字和运算符说明表，供编辑器集成或网站功能使用。

## 数据来源

构建脚本会读取仓库中的 `docs/references` 文档目录，并根据 front-matter 中的 `token`、`title` 等字段生成发布内容。

## 导出内容

- `KEYWORDS`：关键字到 Markdown 文本的映射
- `OPERATORS`：运算符到 Markdown 文本的映射
- `Keyword` / `Operator`：由生成结果推导出的字面量类型

## 安装

```bash
pnpm add @mirascript/help
```

## 示例

```ts
import { KEYWORDS, OPERATORS } from '@mirascript/help';

console.log(KEYWORDS['match']);
console.log(OPERATORS['??']);
```

## 构建

```bash
pnpm --filter @mirascript/help build
```

如果新增或调整 `docs/references` 下的文档，需要重新执行构建以刷新 `dist/` 内容。
