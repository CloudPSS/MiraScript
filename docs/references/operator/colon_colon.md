---
title: '::'
---

`::` 用于扩展调用。

```mira
"Hello"::chars()::join("|")
// 相当于 join(chars("Hello"), "|");
```
