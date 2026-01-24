---
token: '!'
---

`!` 既可以作为前缀逻辑非，也可以作为后缀非空断言。

---

逻辑非：

```mira
!x // x 必须是 boolean
```

---

非空断言：

```mira

let x = nil;
x!; // panic
```
