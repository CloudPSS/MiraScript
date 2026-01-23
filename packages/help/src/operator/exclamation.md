---
token: '!'
order: 26
---

`!` 既可以作为前缀逻辑非，也可以作为后缀非空断言。

- 前缀：`!x`，`x` 必须是 `boolean`
- 后缀：`x!`，当 `x` 为 `nil` 时抛出 `NilError`

```mira
!true; // false

let x = nil;
// x!; // 抛出 NilError
```
