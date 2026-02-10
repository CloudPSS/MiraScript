---
title: '||='
---

`||=` 是短路复合赋值，只有当左侧为 `false` 时才会计算并赋值右侧。

```mira
let mut b = false;
b ||= true;
// b == true
```
