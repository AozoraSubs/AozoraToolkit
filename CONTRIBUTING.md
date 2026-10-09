# CONTRIBUTING.md

本文规定本项目的开发流程、代码风格、测试要求与 Git 约定。

## 1. 变更流程

1. 阅读 `README.md`、`AGENTS.md` 和 `PLAN.md`。
2. 确认本次变更对应的 PLAN 任务，并遵循里程碑依赖关系。
3. 完成并验证工作后，更新 `PLAN.md` 的复选框；未验证的任务不得标记完成。
4. 检查工作区状态与现有实现。
5. 明确本次变更的单一目标。
6. 实现最小可用改动，并添加或更新测试。
7. 运行适用的格式、静态检查和测试。
8. 检查差异、文档和潜在隐私信息。
9. 只有在用户要求提交时才创建 commit。

不要将无关格式化、重命名和功能变更混在同一个提交中。

## 2. Conventional Commits

提交标题使用英文，格式：

```text
<type>(optional scope): <imperative summary>
```

常用类型：

- `feat`: 新功能
- `fix`: 修复缺陷
- `docs`: 文档
- `refactor`: 不改变外部行为的重构
- `perf`: 性能改进
- `test`: 测试
- `build`: 构建系统或依赖
- `ci`: 持续集成
- `chore`: 其他维护任务

示例：

```text
docs: add project architecture and contribution rules
feat(probe): inspect video and audio streams
feat(encoder): add x265 quality settings
fix(jobs): clean up partial output after cancellation
test(progress): cover missing out_time fields
```

要求：

- 标题简短、具体、英文书写。
- 一次提交只对应一个逻辑变更。
- 避免 `update stuff`、`fix bugs` 等模糊标题。
- 有必要时在提交正文说明动机、行为变化和兼容性影响。

## 3. Git commit 签名

本项目不要求使用 OpenPGP 或其他形式的 Git commit 签名。不要为了本项目擅自更改用户已有的 Git 签名配置。如果用户的本机环境已启用自动签名，提交时按其现有配置工作即可。

## 4. 必须执行的检查

执行前先确认仓库实际配置了哪些脚本。不要因 README 中列出了命令就假设脚本一定存在。

Rust 检查：

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

前端依赖与脚本统一使用 pnpm。检查命令以 `package.json` 中实际配置的 scripts 为准，通常包括：

```bash
pnpm install --frozen-lockfile
pnpm run check
pnpm run lint
pnpm run format:check
pnpm test
```

不要使用 npm 或 Yarn 安装依赖，也不要新增第二种包管理器的锁文件。项目应在 `package.json` 的 `packageManager` 字段中固定经过验证的 pnpm 版本。

差异检查：

```bash
git diff --check
git status --short
git diff
git diff --cached
```

如果某项检查不适用或无法运行，说明原因，不得声称检查通过。

## 5. 测试约定

- 修复 bug 时尽可能添加能复现旧问题的测试。
- 新的参数生成逻辑必须测试正常值、边界值和非法值。
- 路径处理应覆盖空格、Unicode、括号和目标文件冲突。
- 进程管理应覆盖成功、非零退出、用户取消和部分输出清理。
- 媒体集成测试使用小型、合法且可再分发的素材。
- 不在自动化测试中依赖特定 NVIDIA GPU；硬件编码测试应作为可选测试，并检测环境能力。
- 测试日志不要包含个人媒体文件名或敏感路径。

## 6. 依赖管理

- pnpm 是本项目唯一受支持的 JavaScript 包管理器。
- 在 `package.json` 的 `packageManager` 字段中固定 pnpm 版本；不要使用 `latest` 作为项目版本声明。
- 提交 `pnpm-lock.yaml`；校验锁文件一致性时使用 `pnpm install --frozen-lockfile`。
- 不提交 `package-lock.json`、`yarn.lock` 或 `npm-shrinkwrap.json`。
- 优先复用现有依赖。
- 引入新依赖前说明解决的问题、替代方案和维护成本。
- 提交锁文件变化。
- 不加入与项目规模不相称的框架。
- 第三方媒体工具与编码器的分发许可证需要单独审查。

## 7. Pull Request / 变更说明模板

```markdown
## Summary
- ...

## Validation
- [ ] Format / lint
- [ ] Unit tests
- [ ] Integration tests (if applicable)

## Risks / Notes
- ...
```

变更说明必须准确反映实际运行的检查和已知限制。
