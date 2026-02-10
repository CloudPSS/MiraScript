---
title: 'case'
---

`case` 是 `match` 表达式中的分支。

```mira
fn age_group(age) {
  match age {
    case 0..12 { "child"}
    case 13..19 { "teenager" }
    case 20..64 { "adult" }
    case _ { "senior" }
  }
}
```
