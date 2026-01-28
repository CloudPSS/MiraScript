---
token: 'break'
---

`break` 用于提前退出 `for` / `while` / `loop`。

---

基本用法

```mira
let mut i = 0;
loop {
  i += 1;
  break;
}
```

---

从循环中返回一个值

```mira
let mut i = 0;
let result = loop {
  i += 1;
  if i == 10 {
    break i * 2;
  }
};
// result 的值是 20
```
