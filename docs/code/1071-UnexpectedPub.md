# UnexpectedPub

等级：**错误**

在模块声明之外出现意外的 `pub`

## 如何修复

`pub` 只能用于模块内部需要导出的声明。

### 修改前

```mira
pub let version = "1.0";
```

### 修复后

```mira
mod config {
  pub let version = "1.0";
}
```
