# @mirascript/constants

`@mirascript/constants` 导出 MiraScript 编译器常量与辅助类型，包括关键字、诊断码、操作码以及配置相关枚举转换逻辑。

## 主要内容

- `DiagnosticCode`
- `OpCode`
- `DIAGNOSTIC_MESSAGES`
- `KEYWORDS` 及相关关键字分类
- `Config`、`InputMode`、`DiagnosticPositionEncoding`
- `isKeyword()` 等辅助函数

## 安装

```bash
pnpm add @mirascript/constants
```

## 示例

```ts
import { KEYWORDS, isKeyword, DiagnosticCode } from '@mirascript/constants';

console.log(KEYWORDS.includes('let'));
console.log(isKeyword('match'));
console.log(DiagnosticCode.InvalidKeyword);
```

## 构建说明

该包会先从 Rust 常量 crate 生成 WebAssembly 产物，再在 JavaScript 层补充类型与工具方法。

```bash
pnpm --filter @mirascript/constants wasm
pnpm --filter @mirascript/constants build
```

构建需要本地安装 `wasm-pack`。
