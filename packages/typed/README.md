# @mirascript/typed

`@mirascript/typed` 提供解析 MiraScript 类型定义的能力。

## MiraScript 类型定义概览

- 内置类型：`nil` `string` `number` `boolean` `record` `array`
  - 其中 `record` `array` 支持泛型：`record<fieldType>` `array<elementType>`
- 联合类型：`typeA | typeB`
  - 可选前缀 `|`
- 数组类型：`type[]`（同 `array<type>`）
- 字面量类型：`"value"` `'value'` `true` `false`
- 记录类型：`(fieldA: typeA, fieldB: typeB)`
  - 可选尾随 `,`
  - 空记录：`()`
  - 匿名字段：`(typeA, typeB)` （只包含一个匿名字段时尾随 `,` 不能省略 `(type,)`）
  - 可选属性：`(field?: type)`
  - 支持使用字符串做字段名以包含特殊字符：`("field-name": type)`
- 函数类型：`fn(arg: type, ..rest: type) -> returnType`
  - 无返回值：`fn(arg: type)`
  - 无参数：`fn() -> returnType`
  - 支持嵌套，例如函数作为参数：`fn(arg: number, callback: fn(result: string, error: any) -> any)`
  - 支持泛型：`fn<T, U>(arg: T) -> U`

## 安装

```bash
pnpm add @mirascript/typed
```

## 基本示例

```ts
import { parse, toJSONSchema } from '@mirascript/typed';

const type = parse('record<number>');
console.log(toJSONSchema(type));
// { type: 'object', additionalProperties: { type: 'number' } }
```

## 常用导出

- `parse()`：将类型字符串解析为 `Type` 对象
- `toJSONSchema()`：将 `Type` 对象转换为 JSON Schema
- `Type`、`KnownType`、`RecordType` 等类型定义

## 开发

```bash
pnpm --filter @mirascript/typed build
pnpm --filter @mirascript/typed test
```
