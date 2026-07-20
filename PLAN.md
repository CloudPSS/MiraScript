# Arena 编译器重构计划

> **状态**: 已实施（见文末"实施备忘"）  
> **预估工作量**: 2-3 个完整工作日  
> **影响范围**: ~55 文件, ~100+ 处 `Box` 替换, 303 编译错误（替换后）

---

## 1. 目标

用 Bump Allocation Arena（`bumpalo`）替代编译器中大量 `Box` 堆分配，消除 AST 构造过程中数百次小对象 `malloc`，预期提升编译吞吐量 **10-30%**，并改善 CPU 缓存局部性。

---

## 2. 当前架构分析

### 2.1 编译器管线

```
Source → Lexer (tokens) → Parser (AST) → Emitter (bytecode)
                                  ↓
                            Formatter (代码格式化, 可选 feature)
```

### 2.2 AST 类型体系

所有 AST 类型位于 `crates/core/src/parser/`，共享一个源生命周期 `'s`（借用 token 流）：

| 类型                          | 文件                | 递归字段（`Box`）                                                                |
| ----------------------------- | ------------------- | -------------------------------------------------------------------------------- |
| `Expression<'s>`              | `expression.rs`     | ~20 个 variant，各含 0-3 个 `Box<Expression>` / `Box<Pattern>` / `Box<Iterable>` |
| `Statement<'s>`               | `statement.rs`      | ~10 个 variant，含 `Box<Expression>` / `Box<Pattern>`                            |
| `Pattern<'s>`                 | `pattern.rs`        | ~10 个 variant，含 `Box<Pattern>`                                                |
| `Callable<'s>`                | `expression.rs`     | 含 `Box<Expression>`                                                             |
| `ElseBlock<'s>`               | `expression.rs`     | 含 `Box<Expression>`                                                             |
| `MatchCase<'s>`               | `expression.rs`     | 含裸 `Expression`、`Pattern`                                                     |
| `Range<'s>`                   | `range.rs`          | 含 2 个 `Box<Expression>`                                                        |
| `Iterable<'s>`                | `iterable.rs`       | 含 `Range`、`Expression`                                                         |
| `ParameterList<'s>`           | `parameter_list.rs` | 含 `Vec<ArrayPattern>`                                                           |
| `ListItem<'s, T>`             | `list_item.rs`      | 含 `Box<T>`（泛型）                                                              |
| `RecordElementBase<'s, E, I>` | `record_element.rs` | 含 `Box<E>`、`Box<I>`（泛型）                                                    |
| `ArrayElementBase<'s, E, S>`  | `array_element.rs`  | 含 `Box<E>`、`Box<S>`（泛型）                                                    |
| `Script<'s>`                  | `script.rs`         | 含 `Vec<Statement>`、`Option<Box<Expression>>`                                   |

### 2.3 关键消费者

| 模块          | 文件                                                                                                     | 访问方式                        |
| ------------- | -------------------------------------------------------------------------------------------------------- | ------------------------------- |
| **Parser**    | `basic_expressions.rs`, `block_expressions.rs`, `statements.rs`, `patterns.rs`, `scripts.rs` 等 15+ 文件 | 构造 AST，返回 owned 类型       |
| **Emitter**   | `emitter_expression.rs`, `emitter_statement.rs`, `emitter_pattern.rs` 等                                 | `&'s Expression<'s>` 引用遍历   |
| **Formatter** | `formatter/expression.rs`, `formatter/statement.rs` 等                                                   | `&Expression<'_>` 引用遍历      |
| **AstWalker** | `ast_visitor.rs`                                                                                         | Trait 抽象，collect diagnostics |
| **Compiler**  | `compile/mod.rs`                                                                                         | 组装 lex → parse → emit 管线    |

---

## 3. 技术方案

### 3.1 选定方案: `bumpalo::boxed::Box<'a, T>` + `'a` 生命周期传播

```rust
// Before
enum Expression<'s> {
    Prefix(TokenRef<'s>, Box<Expression<'s>>),
}

// After
enum Expression<'s, 'a> {
    Prefix(TokenRef<'s>, bumpalo::boxed::Box<'a, Expression<'s, 'a>>),
}
```

**理由**:

- `bumpalo::boxed::Box` 实现 `Deref`、`Clone`、`Debug`、`PartialEq`——保留所有现有 derive
- 通过 `AstArena::alloc()` 分配，返回 arena 生命周期的 Box
- `'a` 参数机械性传播到所有包含递归字段的类型

### 3.2 已排除的方案

| 方案                              | 排除原因                                                             |
| --------------------------------- | -------------------------------------------------------------------- |
| `&'a mut T` 引用                  | `&mut T` 不实现 `Clone`，导致所有 AST 类型的 `#[derive(Clone)]` 失败 |
| `bumpalo::boxed::Box<'static, T>` | `T` 内含 `TokenRef<'s>`（非 `'static`），编译失败                    |
| Index-based Arena (`ExprId`)      | 所有消费者需透过 Arena 间接访问，改动量更大且降低可读性              |

### 3.3 Arena 定义

```rust
// crates/core/src/arena.rs
use bumpalo::Bump;

pub struct AstArena {
    bump: Bump,
}

impl AstArena {
    pub fn new() -> Self { Self { bump: Bump::new() } }

    pub fn alloc<'a, T>(&'a self, value: T) -> bumpalo::boxed::Box<'a, T> {
        bumpalo::boxed::Box::new_in(value, &self.bump)
    }

    pub fn reset(&mut self) { self.bump.reset(); }
}
```

**依赖**: `bumpalo = { version = "3", features = ["boxed"] }` 加入 `Cargo.toml`

### 3.4 `TokenRef` — 不做 Arena 化

`TokenRef<'s>` 保持原有 `Box<Token<'s>>` / `&'s Token<'s>` 双态设计。理由:

- `TokenRef::Owned` 仅在错误恢复路径（如缺失 token 插入）使用，频率极低
- arena 化的性能收益主要在递归 AST 节点上

---

## 4. 变更清单

### 4.1 阶段 1: 基础设施（3 文件）

- [ ] **`crates/core/Cargo.toml`**: 添加 `bumpalo = { version = "3", features = ["boxed"] }`
- [ ] **`crates/core/src/arena.rs`** (新建): `AstArena` 定义（见 3.3）
- [ ] **`crates/core/src/lib.rs`**: 添加 `pub mod arena;` 和 `pub use arena::AstArena;`

### 4.2 阶段 2: AST 类型重构（~12 文件）

所有修改遵循同一模式：类型增加 `'a`，递归 `Box<X<'s>>` → `bumpalo::boxed::Box<'a, X<'s, 'a>>`。

| 文件                       | 变更                                                                                                 |
| -------------------------- | ---------------------------------------------------------------------------------------------------- |
| `parser/expression.rs`     | `Expression<'s, 'a>`, `Callable<'s, 'a>`, `ElseBlock<'s, 'a>`, `MatchCase<'s, 'a>` — 所有 `Box` 替换 |
| `parser/statement.rs`      | `Statement<'s, 'a>` — 所有 `Box` 替换                                                                |
| `parser/pattern.rs`        | `Pattern<'s, 'a>` — 所有 `Box` 替换                                                                  |
| `parser/range.rs`          | `Range<'s, 'a>` — `Box` 替换                                                                         |
| `parser/iterable.rs`       | `Iterable<'s, 'a>` — 内部类型更新                                                                    |
| `parser/parameter_list.rs` | `ParameterList<'s, 'a>`                                                                              |
| `parser/script.rs`         | `Script<'s, 'a>`                                                                                     |
| `parser/list_item.rs`      | `ListItem<'s, 'a, T>` — 泛型 `Box<T>` 替换，`new()` / `new_with_comma()` 签名变更                    |
| `parser/record_element.rs` | `RecordElementBase<'s, 'a, E, I>` — 泛型 `Box` 替换                                                  |
| `parser/array_element.rs`  | `ArrayElementBase<'s, 'a, E, S>` — 泛型 `Box` 替换，类型别名更新                                     |
| `parser/ast_visitor.rs`    | `AstWalker<'s, 'a>` trait + blanket impls（`Vec`, `Option`, `bumpalo::Box`）                         |
| `parser/token_ref.rs`      | `AstWalker` impl 签名更新（类型本身不变）                                                            |

### 4.3 阶段 3: Parser 适配（~15 文件）

- [ ] 所有 parser 函数签名添加 `'a`: `fn foo<'s>(...)` → `fn foo<'s, 'a>(...)`
- [ ] 返回值类型添加 `'a`: `Result<Expression<'s>>` → `Result<Expression<'s, 'a>>`
- [ ] `Box::new(...)` → `arena.alloc(...)`（arena 引用通过参数传入）
- [ ] `ListItem::new()` / `ListItem::new_with_comma()` 调用适配新签名
- [ ] 辅助类型（`Call`、`AccessIndex`、`Function`、`JsonFieldName`、`JsonElement`）添加 `'a`

**关键文件**: `basic_expressions.rs`, `block_expressions.rs`, `statements.rs`, `patterns.rs`, `scripts.rs`, `expressions.rs`, `helper.rs`, `json_expressions.rs`, `array_helper.rs`, `record_helper.rs`, `parameter_list.rs`, `mod.rs`

### 4.4 阶段 4: Emitter 适配（~6 文件）

- [ ] `Emitter<'s, 'c>` → `Emitter<'s, 'c, 'a>` 添加 arena 生命周期
- [ ] 方法签名中的 `&'s Expression<'s>` → `&'s Expression<'s, 'a>` 等
- [ ] `AstWalker<'s, 'a>` trait bound 更新
- [ ] `emit()` 入口函数签名更新

**关键文件**: `emitter/mod.rs`, `emitter_struct.rs`, `emitter_closure.rs`, `emitter_expression.rs`, `emitter_statement.rs`, `emitter_pattern.rs`

### 4.5 阶段 5: Formatter 适配（~10 文件，feature gated）

- [ ] `FormatOptions` 和入口函数签名添加 `'a`
- [ ] `Formattable` trait 及所有 impl 签名更新
- [ ] 内部辅助类型更新

**关键文件**: `formatter/mod.rs`, `formatter/expression.rs`, `formatter/statement.rs`, `formatter/pattern.rs` 等

### 4.6 阶段 6: Compiler 入口 + 公共 API（~3 文件）

- [ ] **`compile/mod.rs`**: `Compiler::compile()` 中创建 `AstArena`，传入 parser；更新 `parse()` 和 `emit()` 调用
- [ ] **`lib.rs`**: 公共导出类型签名更新
- [ ] **Benchmark** (`benches/compile.rs`): 适配新签名

### 4.7 阶段 7: 验证

- [ ] `cargo check` — 编译通过
- [ ] `cargo test --all` — 核心测试通过
- [ ] `cargo test --features formatter` — formatter 测试通过
- [ ] `pnpm --filter @mirascript/mirascript... build && pnpm --filter @mirascript/mirascript test` — e2e 测试通过
- [ ] `cargo bench -p mira-core --bench main` — benchmark 对比（记录重构前后数据）

---

## 5. 风险与缓解

| 风险                             | 缓解                                                       |
| -------------------------------- | ---------------------------------------------------------- |
| `'a` 传播过于广泛，公共 API 破坏 | AST 类型不暴露给下游 binding（napi/python/wasm），影响可控 |
| 编译错误过多，难以定位           | 分阶段推进，每阶段 `cargo check` 验证后再继续              |
| `Clone` derive 失败              | `bumpalo::boxed::Box` 实现 `Clone`，已验证可行             |
| 性能回退                         | 事后 benchmark 对比；mimalloc 可保留作为非 AST 分配的补充  |

---

## 6. 备忘

- **不要**修改 `TokenRef` — 保持 `Box<Token<'s>>`
- **不要**使用 `&'a mut T` — `Clone` 不兼容
- **不要**使用 `bumpalo::boxed::Box<'static, T>` — `TokenRef<'s>` 不满足 `'static`
- `AstArena` 在 `Compiler::compile()` 中创建，单次编译结束后自然 drop
- ~~`bumpalo::boxed::Box` 的 `Clone` 实现会将 clone 分配到**同一** arena~~（已证伪，见下）

---

## 7. 实施备忘（2026-07-20）

- **`bumpalo::boxed::Box` 不实现 `Clone`**（bumpalo 3.x 实际无 `Clone`/`CloneIn`，clone 需要 arena 而 `Clone::clone` 拿不到）。所有 AST 类型的 derive 从 `Debug, Clone, PartialEq` 改为 `Debug, PartialEq`；仅有的 2 处 AST 深拷贝（`parameter_list.rs`、`patterns.rs` spread 错误恢复）改用 `std::mem::replace` 移出原值，语义等价。
- bumpalo Box 不支持 `*box` 移出，改用 `ABox::into_inner()`。
- parser 工厂函数生命周期写为 `<'s: 'a, 'a>`（`ABox<'a, T>` 要求 `T: 'a`）。
- PLAN 遗漏的 `crates/wasm/src/monaco.rs` 已适配：`MonacoCompiler` 新增 `arena: Option<AstArena>` 字段，沿用现有 `'static` unsafe 自引用模式。
- 性能结论：bench 前后基本持平（compile median 59.19 µs → ~58.7-60.0 µs），未达预期 10-30%，原因可能是 mimalloc 全局分配器掩盖了小对象分配收益。
