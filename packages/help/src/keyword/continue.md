---
token: 'continue'
order: 22
---

`continue` 用于跳过本次循环剩余部分，进入下一次循环。

```mira
let mut sum = 0;
for i in 1..5 {
  if i % 2 == 0 { continue; }
  sum += i;
}
// sum == 1 + 3 + 5
```
