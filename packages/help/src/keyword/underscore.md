`_` 是“丢弃”标识符（discard）。常用于：

- 忽略不需要的值
- 在模式匹配/参数模式中表示“我不关心这部分”

```mira
let (_, y) = (1, 2);
// y == 2

fn head { match it { case [x, .._] { x } case _ { nil } } }
```
