---
title: 'not'
---

`not` 用于在模式匹配中取反一个模式，也可以作为 `!` 运算符的替代写法。

---

逻辑非模式：

```mira
fn is_not_weekend(day) {
  day is not 6 and not 7
}
```

---

逻辑非运算符：

```mira
fn is_minor(age) {
  // 等同于 !(age >= 18)
  not (age >= 18)
}
```
