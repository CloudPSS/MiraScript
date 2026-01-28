---
token: 'and'
---

`and` 用于在模式匹配中组合多个模式，也可以作为 `&&` 运算符的替代写法。

---

逻辑与模式：

```mira
fn is_month(month) {
  month is >= 1 and <= 12
}
```

---

逻辑与运算符：

```mira
fn is_adult(age) {
  // 等同于 age >= 18 && age <= 65
  age >= 18 and age <= 65
}
```
