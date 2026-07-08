# @mirascript/typed

`@mirascript/typed` 用于解析 MiraScript 类型定义，并可将结果转换为 JSON Schema。

MiraScript 本身不执行编译期或运行时类型检查。该包主要用于文档、提示系统和结构化数据校验场景。

## 安装

```bash
pnpm add @mirascript/typed
```

## 快速开始

```ts
import { parse, toJSONSchema } from '@mirascript/typed';

const type = parse('record<number>');
console.log(toJSONSchema(type));

console.log(toJSONSchema(parse('(a: number)'), { loose: false }));
console.log(toJSONSchema(parse('(a: number)'), { loose: true }));
```

## 类型语法概览

- 内置类型：`nil` `string` `number` `boolean` `record` `array`
  - `array` 支持泛型：`array<elementType>`
  - `record` 支持泛型：`record<valueType>` 与 `record<keyType, valueType>`
- 交叉类型：`typeA & typeB`
  - 可选前缀 `&`
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
  - 参数类型可省略，默认为 `any`：`fn(a, b: number, ..rest)`
  - rest 参数名可省略：`fn(..) -> string`
  - rest 参数必须是最后一个参数
  - 支持嵌套，例如函数作为参数：`fn(arg: number, callback: fn(result: string, error: any) -> any)`
  - 支持泛型：`fn<T, U>(arg: T) -> U`，同名泛型参数在不同作用域中为不同 symbol
  - 顶层函数可携带函数名：`fn fnName<T>(arg: T) -> any`
- 字符串插值类型：三种引号均支持 `$()` 插值
  - 示例：`` `hello $(type)` ``、`"value: $(string | number)"`
  - `$` 必须紧接 `(`，否则报错；`\$` 可表示字面量 `$`

## 常用导出

- `parse()`：将类型字符串解析为 `Type`
- `toJSONSchema()`：将 `Type` 转换为 JSON Schema
- `Type`、`KnownType`、`RecordType`、`FunctionType`、`TemplateType`、`GenericType` 等类型

## 开发

```bash
pnpm --filter @mirascript/typed build
pnpm --filter @mirascript/typed test
```
