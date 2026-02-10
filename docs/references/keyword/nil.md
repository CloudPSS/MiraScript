---
title: 'nil'
---

`nil` 表示“空值”。当访问不存在的字段、对 `nil` 进行空安全链式调用等场景时，结果也会是 `nil`。

```mira
let x = nil;
let y = x ?? 0;     // 0
let z = x.anything; // nil（空安全访问）
```
