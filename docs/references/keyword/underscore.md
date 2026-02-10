---
title: '_'
---

`_` 是弃元关键字，用于在模式匹配中匹配并忽略任意值。

```mira
let (_, y) = (1, 2); // y == 2

fn head {
  match it {
    case [x, ..] { x }
    case _ { nil }
  }
}
```
