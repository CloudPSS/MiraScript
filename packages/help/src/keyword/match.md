---
token: 'match'
---

`match` 表达式对一个值进行分支匹配，由多个 `case` 组成。

```mira
fn classify {
  match it {
    case (0, 0) { "Origin" }
    case (x, 0) if x > 0 { "Positive X-Axis (x=$x)" }
    case (x, 0) if x < 0 { "Negative X-Axis (x=$x)" }
    case (0, y) { "Y-Axis (y=$y)" }
    case (x, y) { "Somewhere else (x=$x, y=$y)" }
    case _ { "Not a point" }
  }
}
```
