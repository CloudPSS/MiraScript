# ConstantInBindPattern

等级：**错误**

名称以 `@` 开头的变量不允许被重绑定

## 如何修复

不要重绑定 `@` 常量；需要变化的值应声明为可变变量。

### 修改前

```mira
let mut @LIMIT = 10;
```

### 修复后

```mira
let mut limit = 10;
limit = 20;
```
