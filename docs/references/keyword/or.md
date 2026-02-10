---
title: 'or'
---

`or` 用于在模式匹配中组合多个模式，也可以作为 `||` 运算符的替代写法。

---

逻辑或模式：

```mira
fn is_weekend(day) {
  day is 6 or 7
}
```

---

逻辑或运算符：

```mira
fn is_superuser(role) {
  // 等同于 role == "admin" || role == "superuser"
  role == "admin" or role == "superuser"
}
```
