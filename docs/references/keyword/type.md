---
title: 'type'
---

`type` 用于获得值的类型名字符串。

```mira
type(1)       // "number"
```

```mira
nil::type()   // "nil"（扩展调用写法）
```

---

`type` 是非保留关键字，因此可以作为标识符使用。

```mira
let type = fn { 1 }; // 不支持 fn type { .. }
type!() // 1 （通过添加 ! 避免与 type 关键字混淆）
```
