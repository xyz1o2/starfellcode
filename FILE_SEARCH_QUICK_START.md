# 文件搜索快速开始

## 功能概览

输入 `@` 时，自动显示**实时文件搜索建议**。支持：
- ✅ 递归搜索整个项目树
- ✅ 多关键词全文检索
- ✅ 智能排序（文件名优先）
- ✅ 快速导航和选择

## 使用流程

### 1. 激活搜索

输入 `@` 字符：

```
输入框: @
       ↓
显示: 📁 文件建议
     ├─ @src/main.rs
     ├─ @src/app.rs
     ├─ @src/ui/mod.rs
     └─ ... (最多 20 个结果)
```

### 2. 实时过滤

继续输入以过滤结果：

```
输入框: @src/ui
       ↓
显示: 📁 文件建议
     ├─ @src/ui/mod.rs
     ├─ @src/ui/command_hints.rs
     ├─ @src/ui/mention_suggestions.rs
     └─ @src/ui/file_search.rs
```

### 3. 导航选择

使用键盘导航：

| 按键 | 功能 |
|------|------|
| ↑ | 向上选择 |
| ↓ | 向下选择 |
| Enter | 选中并插入 |
| Esc | 关闭建议 |

### 4. 插入文件路径

按 Enter 选中文件：

```
输入框: @src/ui/mod
       ↓ (按 Enter)
输入框: @src/ui/mod.rs
       ↓ (继续输入消息)
输入框: @src/ui/mod.rs What's in this file?
```

## 搜索技巧

### 按目录搜索

```
@src          # 所有 src 目录下的文件
@src/ui       # src/ui 目录下的文件
@src/ai       # src/ai 目录下的文件
```

### 按文件名搜索

```
@main         # 包含 "main" 的文件
@handler      # 包含 "handler" 的文件
@config       # 包含 "config" 的文件
```

### 按扩展名搜索

```
@.rs          # 所有 Rust 文件
@.md          # 所有 Markdown 文件
@.toml        # 所有 TOML 文件
```

### 多关键词搜索

```
@src ui       # 同时包含 "src" 和 "ui" 的文件
@src main     # 同时包含 "src" 和 "main" 的文件
@ai client    # 同时包含 "ai" 和 "client" 的文件
```

## 搜索排序规则

结果按以下优先级排序：

1. **文件名匹配** - 最高优先级
   - `@main.rs` → `src/main.rs` 排在最前

2. **路径位置** - 位置越前得分越高
   - `@src/main` → `src/main.rs` 排在 `src/app/main.rs` 前

3. **多关键词** - 所有关键词得分累加
   - `@src ui` → 同时包含两个关键词的文件排在前面

## 实际使用场景

### 场景 1：查询文件内容

```
输入: @src/app.rs What does this file do?
发送给 AI: 
  用户消息: What does this file do?
  文件内容: <file_content path="src/app.rs">...</file_content>
```

### 场景 2：获取代码建议

```
输入: @src/ui/mod.rs How can I improve this UI code?
发送给 AI:
  用户消息: How can I improve this UI code?
  文件内容: <file_content path="src/ui/mod.rs">...</file_content>
```

### 场景 3：调试问题

```
输入: @src/events/handler.rs Why is the event handler not working?
发送给 AI:
  用户消息: Why is the event handler not working?
  文件内容: <file_content path="src/events/handler.rs">...</file_content>
```

## 性能

| 操作 | 耗时 |
|------|------|
| 首次激活（构建缓存） | ~50ms |
| 搜索查询 | <1ms |
| 导航 | <1ms |
| 显示结果 | <1ms |

## 常见问题

### Q: 为什么没有显示某个文件？

A: 检查以下几点：
1. 文件是否在 `.gitignore` 中？
2. 文件是否在 `target/` 目录中？
3. 搜索关键词是否正确？

### Q: 如何搜索隐藏文件？

A: 隐藏文件（以 `.` 开头）也会被搜索：
```
@.env         # 搜索 .env 文件
@.gitignore   # 搜索 .gitignore 文件
```

### Q: 支持正则表达式吗？

A: 当前不支持，但可以使用多关键词搜索：
```
@test utils   # 查找同时包含 "test" 和 "utils" 的文件
```

### Q: 如何清除搜索？

A: 按 Backspace 删除 `@` 符号，或按 Esc 关闭建议。

## 下一步

- 异步搜索（避免 UI 卡顿）
- 搜索历史
- 文件预览（大小、修改时间）
- 正则表达式支持
- 自定义忽略规则
