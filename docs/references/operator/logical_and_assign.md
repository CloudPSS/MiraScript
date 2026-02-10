---
title: '&&='
---

`&&=` 是短路复合赋值，只有当左侧为 `true` 时才会计算并赋值右侧。

```mira
let mut b = true;
b &&= false;
// b == false
```
