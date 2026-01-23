---
token: '.'
order: 10
---

`.` 用于成员访问（空安全）：当左侧为 `nil` 或字段不存在时返回 `nil`。

```mira
let r = (a: 1);
r.a; // 1
r.b; // nil

let arr = [10, 20];
arr.0; // 10
arr.2; // nil
```
