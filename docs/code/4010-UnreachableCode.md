# UnreachableCode

等级：**提示**

此处的代码不可达。

编译器会检查跳转后的代码、所有可达分支均终止的控制流、没有可达出口的循环，以及布尔字面量和逻辑组合确定不会执行的分支。无守卫或守卫恒真的兜底 `case` 后，其他分支也不可达。

## 如何修复

删除不可达代码，或调整跳转、条件和分支顺序，使代码能够执行。

### 修改前

```mira
fn example() {
  return 1;
  debug_print("unreachable");
}

match value {
  case _ { "fallback" }
  case 1 { "one" }
}
```

### 修复后

```mira
fn example() {
  debug_print("reachable");
  1
}

match value {
  case 1 { "one" }
  case _ { "fallback" }
}
```

分析不会追踪变量值或被调用函数的行为；带未知守卫的 `case _ if condition` 不会使后续分支不可达。函数声明会提升，因此位于跳转后的函数声明本身不会被此诊断标记。
