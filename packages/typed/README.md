# @mirascript/typed

`@mirascript/typed` 提供解析 MiraScript 类型定义的能力。

## MiraScript 类型定义概览

- 内置类型：`nil` `string` `number` `boolean` `record` `array`
  <!-- - 其中 `record` `array` 支持泛型：`record<fieldType>` `array<elementType>` -->
- 联合类型：`typeA | typeB`
  - 可选前缀 `|`
- 数组类型：`type[]`（同 `array<type>`）
- 记录类型：`(fieldA: typeA, fieldB: typeB)`
  - 可选尾随 `,`
  - 空记录：`()`
  - 匿名字段：`(typeA, typeB)` （只包含一个匿名字段时尾随 `,` 不能省略 `(type,)`）
  - 可选属性：`(field?: type)`
  <!-- - 支持使用字符串做字段名以包含特殊字符：`("field-name": type)`-->
