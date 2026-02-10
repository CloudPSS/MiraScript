---
title: 'mod'
---

`mod` 用于声明一个模块。

```mira
mod counter {
  pub let mut value = 0;
  pub fn inc(){
    value += 1;
  }
}

debug_print(counter.value); // 0
counter.inc();
debug_print(counter.value); // 1
```
