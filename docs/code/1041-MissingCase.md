# MissingCase

等级：**错误**

语句中缺少 `case`

## 如何修复

`match` 主体中的每个分支都要以 `case` 开头。

### 修改前

```mira
match 1 {
  1 { "one" }
}
```

### 修复后

```mira
match 1 {
  case 1 { "one" }
}
```
