# Aozora Encode

一个现代、轻量、以本地处理为核心的视频压制工具箱。应用通过图形界面组织 FFmpeg / FFprobe 工作流，让用户能够配置编码器、查看进度、管理任务和复用预设，而不必手动维护复杂的命令行。

> 项目状态：规划 / 初期开发阶段。当前文档描述目标，不代表所有功能已经实现。

## 目标

- 提供清晰、现代的桌面 GUI。
- 通过 FFprobe 检测媒体流信息。
- 通过 FFmpeg 执行视频编码、音视频流映射和封装。
- 支持常见软件编码器与可用的 NVIDIA NVENC 编码器。
- 显示任务状态、进度、速度、日志和错误。
- 支持取消任务，以及保存、导入、导出压制预设。
- 默认在本地处理媒体，不上传用户文件。

## 非目标（第一阶段）

- 自研编解码器。
- 云端编码、账号系统或强制联网功能。
- 完整的视频剪辑时间线。
- 内置完整 VapourSynth/Python 环境。
- 复杂节点式滤镜编辑器。
- 在没有实际需求前支持多任务并行编码。

## 技术栈

- **桌面框架：** Tauri 2
- **前端：** Svelte 5 + TypeScript
- **包管理器：** pnpm（唯一受支持的 JavaScript 包管理器）
- **样式：** CSS variables；保持依赖精简
- **应用核心：** Rust
- **媒体探测与编码：** FFprobe / FFmpeg
- **配置和预设：** JSON + Serde
- **测试：** Rust 单元测试、前端测试、媒体集成测试
- **CI：** GitHub Actions

第一阶段不强制引入 SQLite、VapourSynth 或复杂状态管理库。只有当真实需求出现时再增加依赖。

## 核心设计原则

1. **本地优先：** 默认不上传媒体、不要求账号、不发送遥测。
2. **FFmpeg 是执行引擎：** 应用管理参数与任务，不重新实现编码器。
3. **结构化配置：** 内部保存编码意图，再生成经过校验的进程参数。
4. **安全启动进程：** 使用参数数组，不把用户输入拼成 Shell 命令。
5. **参数透明：** 用户可以查看并复制实际生成的命令与日志。
6. **能力检测：** 以用户当前 FFmpeg 构建实际支持的编码器和滤镜为准。
7. **可诊断失败：** 区分配置错误、工具缺失、编码失败、磁盘问题和用户取消。
8. **小步交付：** 每个变更应聚焦、可测试、可审查。

## 计划中的主要功能

- [ ] FFmpeg / FFprobe 路径设置与能力检测
- [ ] 视频、音频、字幕流信息展示
- [ ] x264、x265 与可用 NVENC 编码器
- [ ] CRF / CQ、preset、像素格式等参数配置
- [ ] 输出路径和容器配置
- [ ] 任务队列、进度、日志与取消
- [ ] 预设保存、导入和导出
- [ ] 命令预览与复制
- [ ] 音轨、字幕轨、章节和附件的高级映射
- [ ] 可选的 VapourSynth 工作流集成

## 开发环境

请安装：

- Git
- Rust stable 工具链（`src-tauri/rust-toolchain.toml` 固定 channel）
- Node.js ≥ 22.12 与 pnpm 11.24.0（以 `package.json` 的 `packageManager` 字段为准）
- Tauri 2 所需的系统构建依赖
- FFmpeg / FFprobe（开发期间可使用本机安装版本）

Windows 构建说明：推荐安装 MSVC Build Tools（含 C++ 桌面开发负载）。若使用 GNU 工具链（`x86_64-pc-windows-gnu`），需要额外把 mingw-w64 binutils（`dlltool`、`ar` 等）加入 `PATH`，因为部分依赖的构建脚本会直接调用 `dlltool`；项目已移除桌面版不需要的 `cdylib` crate-type 以避免 GNU 链接器的导出序号上限问题。

## 本地开发

本项目统一使用 pnpm 管理前端依赖（当前固定 `pnpm@11.24.0`，见 `package.json` 的 `packageManager` 字段）。请安装 Node.js ≥ 22.12 和对应版本的 pnpm。

```bash
pnpm install
pnpm tauri dev
```

请勿使用 npm 或 Yarn 安装依赖，也不要提交 `package-lock.json` 或 `yarn.lock`。仓库只保留 `pnpm-lock.yaml` 作为前端依赖锁文件。

前端检查（脚本定义见 `package.json`）：

```bash
pnpm run check        # svelte-check 类型检查
pnpm run lint         # ESLint
pnpm run format:check # Prettier
pnpm test             # Vitest
```

Rust 检查（在 `src-tauri` 目录）：

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

打包验证：

```bash
pnpm tauri build
```

## 编码器与画质说明

编码器的质量参数不可直接横向比较。例如 x264/x265 的 CRF 与 NVENC 的 CQ 并非同一标尺。推荐预设只应作为起点，不得宣称适用于所有片源的“最佳参数”。

应用必须检测实际编码器能力，不得假定任意 FFmpeg 构建都包含 `libx264`、`libx265`、`libass` 或 NVENC 支持。

## 安全与隐私

- 不使用 Shell 字符串拼接执行用户输入。
- 导入预设时验证 schema 和字段范围；预设不得直接执行任意脚本。
- 默认不联网、不收集遥测。
- 日志避免泄露不必要的本地路径；提供清除日志的方式。
- 输出文件应避免覆盖输入文件；编码失败不得伪装成成功。
- 对随应用分发的 FFmpeg 构建和第三方库进行许可证审查。

## 项目规范

请阅读：

- [`PLAN.md`](PLAN.md)：实施里程碑与可勾选任务清单；AI 开发必须以此为依据并持续更新。
- [`AGENTS.md`](AGENTS.md)：AI 编程助手与所有贡献者必须遵守的工作规则。
- [`CONTRIBUTING.md`](CONTRIBUTING.md)：提交、测试、代码风格和变更流程。
- [`docs/architecture.md`](docs/architecture.md)：架构决策（在该文档建立后）。

## 许可证

尚未确定。发布前必须明确项目许可证，并检查随包分发的 FFmpeg、编码器和其他第三方组件的许可证义务。
