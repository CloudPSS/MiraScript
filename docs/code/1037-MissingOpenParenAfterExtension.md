# MissingOpenParenAfterExtension

等级：**错误**

扩展调用必须以参数列表结尾；请在此处添加 `(`

## 如何修复

扩展调用即使没有额外参数也必须写出 `()`。

### 修改前

```mira
" text "::trim;
```

### 修复后

```mira
" text "::trim();
```
