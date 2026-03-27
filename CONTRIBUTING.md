# 贡献指南

感谢您对 kaf-cli 项目的关注！我们欢迎所有形式的贡献。

## 开发流程

### 1. Fork 和 Clone

```bash
# Fork 本仓库到您的 GitHub 账号
# 然后 clone 您的 fork
git clone https://github.com/YOUR_USERNAME/kaf-cli.git
cd kaf-cli
```

### 2. 创建分支

```bash
git checkout -b feature/your-feature-name
# 或
git checkout -b fix/your-bug-fix
```

### 3. 开发

#### 代码规范

- **语言**: Rust 2021 Edition
- **格式**: 使用 `cargo fmt` 自动格式化
- **检查**: 确保 `cargo clippy --lib` 无警告
- **注释**: 使用中文注释
- **测试**: 新功能必须包含测试

#### 提交前检查

```bash
# 格式化代码
cargo fmt

# 运行 clippy 检查
cargo clippy --lib

# 运行所有测试
cargo test

# 运行特定测试
cargo test --test test_phase61_unit
```

### 4. 提交

```bash
git add .
git commit -m "feat: 添加 xxx 功能

详细描述：
- 变更点 1
- 变更点 2

Closes #123"
```

#### 提交信息规范

格式：`<type>: <subject>`

**type 类型:**
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档变更
- `style`: 代码格式（不影响功能）
- `refactor`: 代码重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建过程或辅助工具变更

**示例:**
```
feat: 添加 Markdown 表格支持
fix: 修复章节识别中的索引越界问题
docs: 更新 README 中的配置示例
```

### 5. 推送和 PR

```bash
git push origin feature/your-feature-name
```

然后在 GitHub 上创建 Pull Request。

## PR 模板

```markdown
## 描述
简要描述这个 PR 做了什么。

## 变更类型
- [ ] Bug 修复
- [ ] 新功能
- [ ] 文档更新
- [ ] 性能优化
- [ ] 代码重构

## 测试
- [ ] 已添加单元测试
- [ ] 已添加集成测试
- [ ] 所有测试通过 (`cargo test`)

## 检查清单
- [ ] 代码已格式化 (`cargo fmt`)
- [ ] Clippy 无警告 (`cargo clippy --lib`)
- [ ] 文档已更新
- [ ] CHANGELOG 已更新

## 相关 Issue
Closes #123
```

## 开发环境设置

### 必需工具

```bash
# Rust 工具链
rustup component add clippy
rustup component add rustfmt

# 可选: cargo-expand（用于宏展开调试）
cargo install cargo-expand
```

### 推荐 IDE 配置

**VS Code 扩展:**
- rust-analyzer
- Even Better TOML
- CodeLLDB（调试）

**设置:**
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all"
}
```

## 项目结构

```
src/
├── cli.rs              # CLI 参数
├── config/             # 配置管理
├── model.rs            # 数据模型
├── parser/             # 解析器
│   ├── chapter_detector.rs
│   ├── markdown_parser.rs
│   └── ...
├── converter/          # 格式转换
├── batch/              # 批量处理
├── style/              # 样式系统
├── utils/              # 工具函数
└── error.rs            # 错误定义

tests/                  # 集成测试
docs/                   # 文档
configs/                # 配置示例
```

## 测试指南

### 单元测试

在 `src/` 文件中内嵌测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        assert_eq!(actual, expected);
    }
}
```

### 集成测试

在 `tests/` 目录创建测试文件：

```rust
// tests/test_new_feature.rs
use kaf_cli::*;

#[tokio::test]
async fn test_new_feature_integration() {
    // 测试代码
}
```

### 测试数据

测试用的 TXT/Markdown 文件放在 `tests/` 目录下（已 gitignore）。

## 性能优化

如需进行性能优化，请先使用 `cargo bench` 建立基准：

```bash
cargo bench
```

然后在 `benches/` 目录添加新的基准测试。

## 文档

- 公共 API 需要文档注释（`///`）
- 复杂逻辑需要行内注释
- 更新 README.md 和 USAGE.md
- 更新 CHANGELOG.md

## 发布流程

1. 更新 `CHANGELOG.md`
2. 更新 `Cargo.toml` 中的版本号
3. 创建 Git tag: `git tag v0.x.x`
4. 推送 tag: `git push origin v0.x.x`
5. 在 GitHub 创建 Release

## 问题反馈

发现 bug 或有功能建议？请通过以下方式：

1. **GitHub Issues**: 详细描述问题
2. **Discussions**: 一般性讨论

### Issue 模板

```markdown
## 问题描述
清晰简洁地描述问题。

## 复现步骤
1. 步骤 1
2. 步骤 2
3. 步骤 3

## 预期行为
描述预期发生什么。

## 实际行为
描述实际发生了什么。

## 环境信息
- OS: [例如 Windows 11]
- Rust 版本: [rustc --version]
- kaf-cli 版本: [kaf-cli --version]

## 附加信息
日志、截图等。
```

## 行为准则

- 友善和尊重地对待所有贡献者
- 欢迎新手，耐心回答问题
- 建设性的批评和讨论
- 关注技术本身而非个人

## 联系方式

- GitHub Issues: [项目/issues](https://github.com/your-username/kaf-cli/issues)
- 邮件: your-email@example.com

感谢你的贡献！🎉
