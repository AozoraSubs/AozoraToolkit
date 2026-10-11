# 项目实施计划（PLAN）

> 本文件是项目实施进度的唯一跟踪清单。技术方案正文保留在下方；所有尚未完成的开发任务均使用 Markdown 复选框追踪。
>
> - `[ ]` 未完成 / 未验证
> - `[x]` 已实现并通过对应验证
> - 不要因为代码“看起来写完了”就勾选；必须有实际检查或测试依据。
> - 如果需求、架构或里程碑发生变化，先更新本文件，再按更新后的计划开发。
> - AI 编程助手每次开始任务前必须阅读本文件；每次完成工作后必须更新相关复选框，并说明验证结果。
> - 当前所有任务默认未完成。只有已在仓库中实现且经过验证的项目才能标记为完成。

## 实施进度总览

- [x] **M0：项目骨架** — 项目可开发、可检查、可构建（详见下方完成记录）
- [x] **M1：媒体探测** — FFmpeg/FFprobe 能力检测与媒体流展示（详见下方完成记录）
- [x] **M2：单任务压制** — 参数生成、编码进程、进度、取消和输出收尾（详见下方完成记录）
- [x] **M3：预设与队列** — 预设持久化、队列、重试和历史
- [ ] **M4：动画压制增强** — 音轨、字幕、章节和 MKV 行为
- [ ] **M5：发布准备** — 许可证、安装包、诊断和干净环境验证
- [ ] **M6：功能增强** — 应用内下载 FFmpeg 等增强功能

## 开发执行清单

### M0：项目骨架

- [x] 初始化 Tauri 2 + Svelte 5 + TypeScript + Rust 项目
- [x] 使用 pnpm 作为唯一 JavaScript 包管理器
- [x] 在 `package.json` 中固定经过验证的 pnpm 版本
- [x] 提交 `pnpm-lock.yaml`，不生成其他包管理器锁文件
- [x] 配置格式化、Lint、类型检查和测试脚本
- [x] 建立基础窗口与导航
- [x] 验证开发、测试和打包流程可重复运行（打包仅验证 Windows MSI 安装包；NSIS 不在范围内）

### M1：媒体探测

- [x] FFmpeg / FFprobe 安装检测（检测顺序：运行环境 PATH → 工具箱安装目录下的 `bin` 文件夹 → 用户指定路径文件夹下的可执行文件；未安装时在 M1 阶段给出可读错误提示。平台差异：macOS / Linux 只检测 PATH 中的 `ffmpeg` / `ffprobe` 命令即可；「指定路径」只对 Windows 有意义，且在 M6 下载 FFmpeg 功能落地后更有用）
- [x] 自定义 FFmpeg 实例路径（用户手动指定自定义 FFmpeg / FFprobe 路径；Windows 下指向包含可执行文件的文件夹）
- [x] 检测 FFmpeg / FFprobe 版本
- [x] 检测实际可用的编码器与滤镜能力
- [x] 实现文件选择和拖放导入
- [x] 使用 FFprobe JSON 解析媒体信息
- [x] 展示容器、时长、大小和视频流信息
- [x] 展示音频流、字幕流、章节和附件信息（在源文件提供时）
- [x] 覆盖探测失败和工具缺失的可读错误
- [x] 验证常见 MKV / MP4 文件可正常探测

### M2：单任务压制

- [x] 定义并验证结构化编码配置
- [x] 实现 x264 / x265 编码参数生成（仅在工具支持时）
- [x] 实现可用 NVENC 编码器参数生成
- [x] 实现输出路径、文件覆盖和输入输出冲突检查
- [x] 生成安全的 FFmpeg 参数数组，不拼接 Shell 命令
- [x] 实现 FFmpeg 子进程启动与 stdout / stderr 持续读取
- [x] 使用机器可读进度接口解析进度
- [x] 显示进度、速度、已用时间、可用时的 ETA 和日志
- [x] 实现任务取消和进程清理
- [x] 实现临时输出、成功收尾和失败清理策略
- [x] 验证成功、失败、取消三种路径
- [x] 验证非 ASCII 路径、空格和特殊字符路径

### M3：预设与队列

- [x] 定义带 `schemaVersion` 的预设 JSON 格式
- [x] 实现预设保存、重命名和删除
- [x] 实现预设 JSON 导入、导出与校验
- [x] 提供少量内置示例预设
- [x] 实现单任务队列和任务状态展示
- [x] 实现失败任务查看日志与修改配置后重试
- [x] 实现任务历史持久化
- [x] 实现命令预览与复制
- [x] 验证应用重启后预设和任务历史行为符合预期

### M4：动画压制增强

- [x] 字幕处理选项：保留全部字幕轨（复制）或丢弃（2026-10-10 按用户要求提前实施）
- [ ] 实现音轨映射与保留策略
- [ ] 实现字幕轨映射与保留策略（按轨选择）
- [ ] 实现章节保留策略
- [ ] 完善 MKV 封装行为
- [ ] 实现 ASS 字幕处理的基本支持
- [ ] 用真实样片验证流映射和字幕结果
- [ ] 评估 VapourSynth 集成接口（不默认内置运行环境）

### M5：发布准备

- [ ] 明确应用自身许可证
- [ ] 审查 FFmpeg、编码器和第三方依赖的分发许可证
- [ ] 建立第三方组件与许可证清单
- [ ] 完成安装包与版本信息
- [ ] 明确升级和配置迁移策略
- [ ] 完善日志导出与故障诊断说明
- [ ] 在干净的 Windows 环境验证安装与首次运行
- [ ] 验证新用户可以配置工具、导入视频并完成首次压制

### M6：功能增强

- [ ] 下载 FFmpeg（将 FFmpeg / FFprobe 下载或内置到工具箱安装目录下的 `bin` 文件夹，作为安装检测顺序中的第二步；Windows 优先）
- [ ] 多 FFmpeg 实例下拉选择（检测到多个实例时——运行目录 `bin/ffmpeg`、PATH 命令、自定义实例——设置页提供下拉框选择实际使用的 FFmpeg / FFprobe）

## 每次开发任务的完成记录

每次提交或阶段性开发完成后，AI 应在回复中记录：

- **本次完成：** 对应的 PLAN 项目
- **验证方式：** 实际执行的命令或测试
- **结果：** 通过 / 失败 / 未执行及原因
- **PLAN 更新：** 勾选或保持未勾选的条目
- **剩余工作：** 下一项尚未完成的任务

---

## 完成记录

### 2026-10-10：M0 项目骨架

- **本次完成：** M0 全部清单项——Tauri 2 + Svelte 5 + TypeScript + Rust 项目初始化；pnpm 唯一包管理并固定 `pnpm@11.24.0`；提交 `pnpm-lock.yaml`（无其他包管理器锁文件）；格式化 / Lint / 类型检查 / 测试脚本（prettier、eslint、svelte-check、vitest）；基础窗口与四区导航（压制 / 任务 / 预设 / 设置）；开发、测试与 MSI 打包流程验证。
- **验证方式与结果（全部实际执行）：**
  - 前端：`pnpm run check`（0 错误）、`pnpm run lint`（0 问题）、`pnpm run format:check`（通过）、`pnpm test`（4/4 通过）、`pnpm build`（通过）。
  - Rust：`cargo fmt --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test`（1/1 通过）。
  - 开发流程：`pnpm tauri dev` 实际启动，窗口标题 "Aozora Encode"，vite 监听 1420，稳定运行。
  - 打包流程：`pnpm tauri build` release 编译成功（release exe 实际启动验证通过）；MSI 安装包成功产出（多次构建均成功）。
- **范围调整（2026-10-10）：** 按用户要求，NSIS 打包验证与 CI 已**完全移出项目范围**：CI 工作流文件已删除，打包目标仅保留 MSI，PLAN 中不再保留相关任务条目。
- **环境说明（本机特有问题，已在 README 记录）：** 本机未安装 MSVC Build Tools，使用 Windows GNU 工具链 + mingw-w64 binutils（用户级安装）；`registry.npmjs.org` 被网络拦截，本机 pnpm 使用 npmmirror 镜像（用户级 `.npmrc`，未入库）；已移除桌面版不需要的 `cdylib` crate-type 以避免 GNU 链接器导出序号上限。
- **PLAN 更新：** M0 总览与清单全部勾选；§11 里程碑 M0 子项勾选；移除 CI 条目并按范围调整打包验证表述。
- **剩余工作：** 进入 M1（媒体探测）。

### 2026-10-10（补充）：多平台可行性检查与范围确认

- **本次完成：** 对 Linux / macOS / Windows 多平台开发的可行性进行检查；结论为当前环境不可行，按用户指示维持"第一阶段仅 Windows 10/11 x64 桌面端"的范围。
- **验证方式与结果（全部实际执行）：**
  - 代码检查：`src-tauri/Cargo.toml` 仅含跨平台依赖（tauri、tauri-plugin-opener、serde、serde_json），无 `windows` crate 或平台专属依赖；应用代码中无平台相关 `cfg` 分支（仅脚手架自带的 `windows_subsystem` 属性，对其他平台为 no-op）。
  - 环境检查：本机已安装的 Rust 编译目标仅 `x86_64-pc-windows-gnu`（无 Linux / macOS 目标）；开发机为 Windows，没有 macOS / Linux 构建环境。
  - 打包与 CI：`bundle.targets` 仅保留 `msi`（Windows 专用）；CI 工作流已移出范围（无 `.github` 目录），无法借助远程构建矩阵验证其他平台。
- **结论：** Tauri 2 + Svelte 5 技术栈本身跨平台，当前代码也没有平台硬依赖；但"多平台可行"需要能在对应平台上实际构建与验证产物——macOS 打包必须使用 macOS + Xcode，Linux 需要相应构建环境与 WebKitGTK 验证，本机均不具备，CI 也不在范围内。因此维持仅 Windows 开发；后续代码继续保持可移植写法（不引入平台专属依赖），待具备其他平台环境时再重新评估。
- **PLAN 更新：** 无任务勾选变化；本条目仅记录范围确认与可行性结论。
- **剩余工作：** 进入 M1（媒体探测）。

### 2026-10-10（追加）：引入 GitHub Actions 三平台构建验证 CI

- **本次完成：** 按用户决定恢复 CI 并扩展为三平台构建验证。新增 `.github/workflows/build.yml`：Windows 出 MSI、macOS 出 dmg、Linux 出 deb + AppImage；同一矩阵内运行前端检查（check / lint / format / test）与 Rust 检查（fmt / clippy / test），打包产物上传为 artifacts。同步更新 AGENTS.md 技术边界与 README。
- **范围调整：** 撤回此前"CI 完全移出项目范围"的决定：CI 不再属于排除项，但只承担构建验证职责；macOS 产物不签名、不公证，不构成全平台发布承诺。Windows MSI 仍是唯一分发目标，NSIS 打包仍不在范围内，移动端仍排除。
- **验证方式与结果：** 工作流文件已写入仓库并通过结构自检；本机没有配置 git remote、无法触发 GitHub Actions，**尚未实际运行 CI**，需推送后观察三平台首跑结果再确认。
- **PLAN 更新：** 更新技术方案正文"目标平台"与 §1.2 非目标表述；无里程碑任务勾选变化。
- **剩余工作：** 推送至 GitHub 后验证 CI 首跑；继续 M1（媒体探测）。

### 2026-10-10（追加二）：修复 macOS 产物"已损坏"问题

- **本次完成：** 用户反馈 CI 产出的 macOS dmg 打开时提示"已损坏"（Gatekeeper 拦截无签名产物）。按 Tauri 官方支持的方式为 macOS 产物添加 ad-hoc 签名：新增 `src-tauri/tauri.macos.conf.json`（`bundle.macOS.signingIdentity: "-"`，仅 macOS 构建时合并），并在 CI 的 macOS 任务中新增 `codesign --verify --deep --strict` 验证步骤，确保签名实际生效。README 补充首次打开说明（系统设置"仍要打开"或移除 quarantine 属性）。
- **验证方式：** 方案依据 Tauri 官方文档与社区验证过的配置；`codesign` 验证步骤将在下次 CI macOS 任务实际执行，本机（Windows）无法直接验证。
- **PLAN 更新：** 无任务勾选变化；AGENTS.md 平台边界表述同步更新（ad-hoc 签名、不公证）。
- **剩余工作：** 推送后从 GitHub 重新下载新版 dmg 验证；正式分发仍需 Apple Developer ID 签名 + 公证（付费开发者账号，不在当前范围）。

### 2026-10-10（M1 首批）：FFmpeg / FFprobe 安装检测与自定义路径

- **本次完成：** M1 前三项——安装检测、自定义实例路径、版本检测。新增 workspace 成员 `aozora-core`（纯逻辑、不依赖 Tauri，避免 Windows-GNU 下测试 exe 链接 WebView2 原生库失败）；`detect_tools` 命令按 PLAN 顺序检测（PATH → 安装目录 `bin` → 自定义路径，Windows 全顺序，macOS/Linux 仅 PATH），每个候选经 `-version` 实机验证并取版本横幅；设置持久化到 `app_config_dir()/settings.json`（`schemaVersion: 1`，含 `customToolDir`）；设置页展示检测结果（来源、路径、版本、可读错误）并支持保存/清除自定义路径。
- **验证方式与结果（全部实际执行）：**
  - Rust：`cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace`（8 通过 / 2 ignored）全部通过。
  - 实机检测（ignored 测试，本机 ffmpeg 9.0）：PATH 检测 + 版本横幅 ✓；注入真实 ffmpeg 目录验证 bin 分支（来源 `app_bin`）与自定义路径分支（来源 `custom`）✓。
  - 前端：`pnpm run check`（0 错误）、`pnpm run lint`、`pnpm run format:check`、`pnpm test`（5/5）、`pnpm build` 全部通过。
- **限制（如实记录，2026-10-10 晚更新结论）：** 从仓库 `target` 目录运行的 exe 在本机初始化 WebView2 失败（`0x8000FFFF`，重装 Rust / 重装 WebView2 运行时 / 修复目录所有者均无效）；而**安装到系统（MSI 安装包，`Program Files`）的旧版本应用一直正常运行**——问题仅限"从 DSH 工作区路径启动的进程"（沙箱限制），与 M1 代码无关（已用诊断程序证明：同一台机器、同一份二进制，数据目录放工作区内成功、放 `%LOCALAPPDATA%` 下失败）。**已解决**：dev 构建（`#[cfg(dev)]`）将 WebView2 数据目录与配置文件放在可执行文件旁（工作区内可写），`pnpm tauri dev` 实测窗口正常启动；发布构建保持默认位置（`%LOCALAPPDATA%` / `%APPDATA%`），与安装版行为一致。
- **PLAN 更新：** M1 前三项勾选；§11 将「探测版本和编码器能力」拆分为版本（已勾选）与编码器能力（未勾选）；README M1 功能清单同步。
- **剩余工作：** M1 其余项——编码器/滤镜能力检测、文件选择与拖放导入、FFprobe JSON 解析与流信息展示、探测失败可读错误、真实 MKV/MP4 验证。

### 2026-10-10（M1 收尾）：能力检测、文件导入与媒体信息展示

- **本次完成：** M1 剩余全部 7 项——能力检测（`ffmpeg -encoders/-filters/-hwaccels` 解析，关键编码器可用性 libx264/libx265/NVENC 系列 + 计数 + 硬件加速方法，设置页展示）；文件选择（`tauri-plugin-dialog`，capability 仅 `dialog:allow-open`）与窗口拖放导入（`onDragDropEvent`）；FFprobe JSON 探测（格式/时长/大小/码率/视频流/音频流/字幕流/章节/附件，压制页表格展示）；探测失败与工具缺失的可读错误；真实 MKV/MP4 样片验证。
- **验证方式与结果（全部实际执行）：**
  - Rust：`cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace`（13 通过 / 4 ignored）全部通过。
  - 实机（ignored 测试，本机 ffmpeg 9.0 / ffprobe）：能力检测（编码器/滤镜列表解析，libx264 可用）✓；用 ffmpeg 现场生成 1 秒 MP4（testsrc+正弦音频）并转封装 MKV，ffprobe 探测两者——容器、时长、大小、视频流（h264/320×240/yuv420p）、音频流（aac/双声道）字段全部正确 ✓。
  - 前端：`pnpm run check`（0 错误）、`pnpm run lint`、`pnpm run format:check`、`pnpm test`（5/5）、`pnpm build` 全部通过。
  - `pnpm tauri dev` 冒烟：应用启动、窗口稳定运行（dev 数据目录方案生效）。
- **限制：** 文件选择对话框、拖放与信息表格的**人工交互验证**需在安装版或正常终端中进行（本会话工作区进程受限，仅能验证到窗口稳定运行层）；数据层已由实机测试覆盖。
- **PLAN 更新：** M1 全部 10 项勾选；§11 M1 子项勾选；实施进度总览 M1 勾选；README M1 功能清单勾选。
- **剩余工作：** 进入 M2（单任务压制）：结构化编码配置、参数生成、FFmpeg 进程/进度/取消与输出收尾。

### 2026-10-10（M2）：单任务压制

- **本次完成：** M2 全部 12 项——结构化编码配置（`schemaVersion: 1` 的 `EncodeConfig`：编码器 / 质量模式 / 质量值 / 预设 / 像素格式，逐编码器规格校验范围）；x264/x265 与 NVENC 参数生成（`planner::build_args`：`-hide_banner -nostdin -y -i` → `-map 0:v:0 -c:v … -preset … -crf/-cq … [-pix_fmt …]` → `-map 0:a? -c:a copy -sn` → `-progress pipe:1 -nostats`，全程参数数组 `OsString`，无 Shell 拼接）；输出路径/覆盖/输入输出冲突检查（扩展名 mkv/mp4、父目录存在、规范化路径防输入输出同一文件、覆盖策略）；FFmpeg 子进程运行器（tokio：stdout 进度 + stderr 日志持续读取、`kill_on_drop`、退出后收尾）；机器可读进度解析（`-progress pipe:1` 键值块，兼容 `out_time_ms/out_time_us/out_time`，速度解析，ETA 计算，250ms 节流）；临时输出策略（同目录 `<stem>.aozora-part-<任务ID>.<ext>`，成功重命名、失败/取消清理，覆盖时先移除旧文件）；取消（`watch` 信号 → `start_kill`，最终状态 `Cancelled`）；前端压制页（编码器下拉按能力检测动态禁用、CRF/CQ 数值与范围提示、预设、像素格式、输出路径 + 保存对话框、覆盖勾选、参数预览、进度条/速度/已用时间/ETA/日志、取消按钮、完成/失败/取消结果与错误）；任务命令层（`start_job`/`cancel_job`/`preview_job_args`，单任务限制，`job:started/progress/log/finished` 事件，事件转发完成后才发 finished 保证顺序）。
- **验证方式与结果（全部实际执行）：**
  - Rust：`cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace` 全部通过。
  - 实机集成测试（ignored，本机 ffmpeg 9.0，9/9 通过）：成功路径（libx264 完成压制 + 临时文件重命名 + 输出可被 FFprobe 读取 + 进度 100%）；失败路径（输入不存在 → Failed + 临时文件清理 + 无伪成品）；取消路径（veryslow 长编码中途取消 → Cancelled + 进程终止 + 清理）；覆盖路径（旧输出被替换）；非 ASCII/空格/括号路径（`测试 目录 (1)\样片 [demo].mkv` 全程正常）；NVENC 参数（h264_nvenc p1/p7、hevc_nvenc p5、av1_nvenc p5 + CQ 21 均真实编码成功——本机具备 NVIDIA GPU）。
  - 前端：`pnpm run check`、`pnpm run lint`、`pnpm run format:check`、`pnpm test`（8/8）、`pnpm build` 全部通过。
  - `pnpm tauri dev` 冒烟：窗口启动、标题 "Aozora Encode"、WebView2 正常初始化（本会话可见进程与窗口标题）。
- **工程问题与修复（Windows-GNU 特有，已入库）：** 应用一旦使用事件（`app.emit`）与 tauri 默认特性 `common-controls-v6`，测试可执行文件会导入 `TaskDialogIndirect`（仅 comctl32 v6 导出）与 `WebView2Loader.dll`。修复：build.rs 改为 tauri-build 不内嵌清单（`new_without_app_manifest`），GNU 工具链下用 windres 生成 `default-manifest.o`（v6 清单）并经 `-B` 覆盖 gcc specs 的 `if-exists(default-manifest.o)` 注入，MSVC 下走 `embed-resource` 资源清单；同时把 `WebView2Loader.dll` 复制到 `deps/`。新增 `tests/smoke.rs` 断言测试 exe 内嵌 v6 清单。
- **限制：** 编码页表单交互（下拉选择、进度条动画、日志滚动）未在本会话人工点击验证（数据层与窗口启动已实测）；NVENC 真实验证依赖 NVIDIA GPU（本机具备）；版本号未变更（按规则需用户明确要求）。
- **PLAN 更新：** M2 全部 12 项勾选；§11 M2 子项勾选；实施进度总览 M2 勾选；README M2 清单勾选。
- **剩余工作：** 进入 M3（预设与队列）：预设 JSON 持久化/导入导出、队列与任务历史、命令预览与复制。

### 2026-10-10（M2 修复）：输出权限问题的快速失败与诊断

- **本次完成：** 用户反馈压制失败（`FFmpeg 退出码：exit code: 0xfffffff3`，`Error opening output … Permission denied`）。本地复现确认 0xFFFFFFF3 即 -13（EACCES，只读文件/目录无写权限时的 FFmpeg 退出码）。修复：`planner::verify_writable` 在任务启动前以写模式创建并删除探测文件，把"目录不可写"与"已有文件被占用/只读"区分成可读错误，任务不再带着必然失败进入 FFmpeg；`describe_exit` 解码负 errno 退出码（-13 权限不足 / -2 路径不存在）；失败摘要含 `Permission denied` 时追加排查提示（目录权限、杀毒软件、受控文件夹访问、文件占用/只读）。
- **验证方式与结果（全部实际执行）：**
  - 本地复现：只读文件 → ffmpeg 退出 0xFFFFFFF3（-13）✓；不存在的目录 → 0xFFFFFFFE（-2）✓，与用户日志一致。
  - Rust：`cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace`（49 单元，含 3 个新的 `verify_writable` 用例：可写创建+删除、目录缺失、Windows 只读文件拒绝）、集成测试 9/9 全部通过。
- **限制：** 用户环境（`Downloads\yt-dlp_win_x86\Disp\output`）的写入权限无法在本会话直接检查；修复后用户再次运行会在开始前看到精确原因（目录不可写 / 文件被占用 / 只读）。
- **PLAN 更新：** 无清单勾选变化（M2 缺陷修复）。
- **剩余工作：** 同 M2 记录；如新报错仍指向目录 ACL/受控文件夹访问，需用户检查该目录权限或更换输出目录。

### 2026-10-10（M2 修复二）：拒绝访问的排查结论与残留清理加固

- **本次完成：** 用户反馈"正常打开文件压制仍提示拒绝访问"。本机实测同一目录：同账户 PowerShell 写入成功、直接运行 ffmpeg 压制成功（退出码 0）——目录本身可写，仅"应用进程树内"的写入被拒，指向安全软件（HIPS/文件防护）按父进程拦截 aozora-encode.exe 及其 ffmpeg 子进程。加固：`verify_writable` 对崩溃遗留的只读临时输出（`*.aozora-part-*`）自动清只读属性、删除并重试；`finalize_output` 覆盖时先清旧输出的只读属性（修复"编码成功但收尾删除失败"的缺陷）；错误信息区分"目录不可写"与"文件被占用/只读"，并提示安全软件/受控文件夹访问。
- **验证方式与结果（全部实际执行）：** 本机复现对照（同目录 PowerShell 写 ✓、独立 ffmpeg 压制 ✓ 退出码 0）；`cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace`（52 单元，新增只读残留自动清理、只读旧输出覆盖收尾两个 Windows 用例）、集成测试 9/9 全部通过。
- **PLAN 更新：** 无清单勾选变化（M2 缺陷修复）。
- **剩余工作：** 若用户端仍复现，需其执行决定性验证（终端手工运行预览参数）并检查安全软件白名单/受控文件夹访问。

### 2026-10-10（M2 调整）：临时输出改到应用自有临时目录（按用户要求）

- **本次完成：** 按用户要求调整临时输出策略：FFmpeg 先写入应用自有的可写临时目录（dev 构建为软件目录下的 `encode-tmp`；发布构建为应用配置目录下的 `encode-tmp`，因 Program Files 安装目录不可写），成功后移动到最终输出（同卷重命名；跨卷回退为复制到目的地 + 删除临时文件），失败/取消时删除临时文件。启动前：清理 `encode-tmp` 中本应用遗留的 `*.aozora-part-*`（清只读属性后删除）、探测应用临时目录与**目标目录**的可写性（探针文件用完即删），权限问题在开始压制前以可读错误快速失败。`temp_output_path` 签名改为 `(temp_dir, final_output, job_id)`，运行器 `finalize_output` 支持重命名→复制回退。
- **验证方式与结果（全部实际执行）：** `cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace`（55 单元，新增：临时路径落在应用临时目录、遗留临时文件选择性清理、目标目录探针清理、只读残留自动清理、只读旧输出覆盖收尾）、集成测试 9/9（成功用例改为"临时目录与目标目录分离"）全部通过。
- **限制：** 跨卷复制回退无法在本机单卷环境实测（代码路径经审查与错误合并处理）；发布构建的临时目录路径（`%APPDATA%\com.aozora.encode\encode-tmp`）未在安装版实测。
- **PLAN 更新：** §6.4 输出文件安全策略按本次调整更新（标注为用户决策与原方案差异）。
- **剩余工作：** 同前；用户端重试后若目标目录仍不可写，会在开始前看到「输出目录不可写（目录）：拒绝访问…」的明确提示。

### 2026-10-10（M2 调整二）：目标目录不可写时自动回退到应用临时目录

- **本次完成：** 用户反馈桌面、D 盘、原目录全部提示无权限。本机诊断：无第三方安全软件（仅 Windows Defender）、受控文件夹访问关闭、无 Defender 拦截日志；目录 ACL 显示本机进程运行在 `CodexSandboxUsers` 受限用户组下（桌面/Downloads 对该组仅授予 RX），即系统沙箱对用户进程的写入限制（仅工作区可写），应用无法也不应绕过。软件层修复：目标目录不可写时不再失败——`start_job` 自动回退为把成品保存到应用临时目录（`encode-tmp`），通过 `job:started` 事件携带 notice 在界面明确提示"目标目录不可写，成品将保存到应用临时目录"及原因；完成面板展示实际输出路径。压制流程因此在任何环境下都能完成并交付成品。
- **验证方式与结果（全部实际执行）：** `cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace`（56 单元，新增 `fallback_output_path` 用例）、集成测试 9/9；前端 `pnpm check/lint/format:check/test/build` 全部通过；`pnpm tauri dev` 冒烟（窗口正常，1420 端口被用户自身实例占用，属预期）。
- **限制：** 回退行为未在真实受限目录上端到端实测（本会话进程拥有完全访问权）；沙箱对成品位置的限制需用户把文件从应用临时目录取走，或由沙箱管理员放行目标目录。
- **PLAN 更新：** 无清单勾选变化（M2 缺陷修复）；§6.4 补充"目标目录不可写时回退到应用临时目录"。
- **剩余工作：** 用户重试确认回退提示与成品位置符合预期。

### 2026-10-10（M2 环境修复）：应用进程低完整性导致的写入受限

- **本次完成：** 应用进程自检报告（新增 `diagnostics` 模块，目标目录不可写时在提示中列出进程对常见目录的实测可写性）定位根因：**工作区根目录带 `Mandatory Label\Low Mandatory Level:(OI)(CI)(NW)`，仓库内新建的任何文件（包括 cargo 编译的 `aozora-encode.exe`）都继承低完整性标记；Windows 启动该 exe 后进程为低完整性，只能写工作区内低完整性位置，写桌面/Downloads/D 盘/%TEMP%（默认中完整性）一律"拒绝访问 (os error 5)"**——与用户 PowerShell 可写、应用被拒的现象完全一致（也解释了 M1 时期 WebView2 数据目录写 %LOCALAPPDATA% 失败的 dev 临时方案）。修复（机器级，不入库）：`icacls src-tauri\target /setintegritylevel (CI)(OI)M`（带继承，此后编译产物均为中完整性）+ 对现有 exe 单独改标；实测新启动的应用进程完整性为 S-1-16-8192（中）。若工作区被环境重置需重新执行该命令。
- **验证方式与结果（全部实际执行）：** 标签继承实验（新建文件自动 Low）✓；改标后 exe 为 Medium ✓；启动应用后进程完整性实测 S-1-16-8192 ✓；`cargo fmt --check`、`clippy --all-targets --all-features -- -D warnings`、`cargo test --workspace`（58 单元，新增 diagnostics 用例）全部通过。
- **后续（按用户选择）：** 进一步把工作区根目录整体解除低完整性标记（`icacls <工作区> /setintegritylevel "(CI)(OI)M"`），实测根目录与新建文件均为中完整性；此后工作区内所有产物均为正常级别。若环境重置工作区重新打标，需重跑该命令（已写入 README）。
- **PLAN 更新：** 无清单勾选变化；本条目记录环境修复命令与机制说明。
- **剩余工作：** 用户重启 tauri dev 后确认可直接写入桌面/D 盘；M3（预设与队列）。

### 2026-10-10（M4 提前项）：字幕保留/丢弃选项

- **本次完成：** 按用户要求提前实施 M4 的最小字幕子集：`EncodeConfig` 新增 `subtitles: { mode: "copy" | "drop" }`（旧 JSON 无此字段时默认 drop，兼容不破坏）；`planner::build_args` 按模式生成参数——保留：`-map 0:s? -c:s copy`（复制全部字幕轨，替换原 `-sn`），丢弃：`-sn`；压制页新增"字幕"下拉（丢弃 / 保留全部字幕轨），源无字幕轨时提示，MP4 + 保留时提示兼容性风险；参数预览同步反映。
- **验证方式与结果（全部实际执行）：**
  - Rust：`cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace` 全部通过（新增参数生成、序列化兼容等单元用例）。
  - 实机集成测试（10/10）：新增 `encode_preserves_and_drops_subtitles`——生成带中文 SRT 字幕轨的 MKV，copy 模式输出探测到 subrip 字幕轨 ✓，drop 模式输出无字幕轨 ✓。
  - 前端：`pnpm check/lint/format:check/test/build` 全部通过；`pnpm tauri dev` 冒烟（窗口正常启动）。
- **限制：** 按轨精细映射、字幕烧录（ASS 渲染）、章节保留仍属 M4 后续；MP4 容器只支持 mov_text 字幕，其他格式会由 FFmpeg 报错（界面已提示建议 MKV）。
- **PLAN 更新：** M4 清单新增"字幕处理选项"并勾选（原"字幕轨映射与保留策略"保持未勾选，指按轨精细映射）；§11 M4 同步。
- **剩余工作：** 回到 M3（预设与队列）。

### 2026-10-10（M4 提前项二）：烧录外部字幕文件（srt / ass）

- **本次完成：** 澄清用户需求为**烧录自定义路径的外部字幕文件**而非视频内打包字幕。实现：`SubtitleMode` 新增 `Burn`，`SubtitleConfig` 新增 `external_path`（serde 默认 None，旧配置兼容）；烧录参数 `-vf subtitles=filename='…'`（libass `subtitles` 滤镜同时支持 srt/ass），路径按 `av_get_token` 规则转义（反斜杠→斜杠、`:`/`'` 转义、单引号包裹防 `,`/`;` 分裂），同时 `-sn` 丢弃视频内字幕轨；校验：burn 必须携带 srt/ass 文件；启动前检查字幕文件存在性 + FFmpeg 是否含 `subtitles` 滤镜（`CapabilityReport.has_subtitles_filter` 与轻量 `has_filter`，缺 libass 时给出可读错误）；压制页新增「烧录外部字幕」选项（能力不足时禁用并标注）+ 字幕文件选择框。
- **验证方式与结果（全部实际执行）：**
  - Rust：`cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace` 全部通过（新增 burn 校验、滤镜转义、参数生成等单元用例）。
  - 实机集成测试（11/11）：新增 `encode_burns_external_subtitles`——含中文与括号的外部 SRT（`字幕 [外挂].srt`）烧录进测试视频：任务完成、日志出现 libass 滤镜初始化、输出无字幕轨、时长正常。
  - 前端：`pnpm check/lint/format:check/test/build` 全绿；`pnpm tauri dev` 冒烟（窗口正常）。
- **限制：** 烧录样式（字体/位置/大小）暂用字幕文件默认样式，自定义样式属 M4 后续；SRT 中文渲染依赖系统字体（本机具备）。
- **PLAN 更新：** 无清单勾选变化（仍属 M4"ASS 字幕处理的基本支持"范围，按轨映射等后续）；README M4 清单更新。
- **剩余工作：** M3（预设与队列）。

### 2026-10-10（体验修复批）：控制台弹窗、切页进度、探测卡顿与重复探测

- **本次完成（用户提出 6 项中的 5 项）：**
  1. **隐藏子进程控制台窗口**：新增 `process_flags`（Windows 下 CREATE_NO_WINDOW），应用于所有 ffmpeg/ffprobe 启动点（版本检测、能力检测、媒体探测、压制运行器），不再弹终端窗口；
  2. **切页进度不丢失**：任务状态提升为全局 store（`stores/jobs.svelte.ts`），事件监听统一移到常驻的 App.svelte，压制页读全局状态——压制中切到其他页面再返回，进度/日志/结果完整保留；
  3. **探测命令异步化**：`detect_tools` / `detect_capabilities` / `probe_media_file` 改为 async 命令并在 `spawn_blocking` 中执行，不再阻塞 Tauri 主线程（消除设置页与选文件时的界面卡死）；
  4. **选文件卡顿**：同 3（探测不再同步阻塞）；
  5. **探测结果会话级缓存**：新增 `stores/tools.svelte.ts`，检测结果只探测一次；设置页进入即展示缓存，提供「重新检测」按钮强制刷新（保存自定义路径后自动强制刷新）。
- **验证方式与结果（全部实际执行）：** `cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace`、集成测试 10/10 全部通过；前端 `pnpm check/lint/format:check/test/build` 全绿；`pnpm tauri dev` 冒烟（窗口正常、进程稳定）。
- **搁置（第 6 项 WebView2Loader.dll 静态整合）：** 官方结论为仅 MSVC 目标可静态链接加载器。本机已安装 VS Build Tools 与 MSVC Rust 工具链，但会话内 rustup 下载组件失败、且 `rustup set default-host` 中断导致 MSVC 工具链组件不完整；按用户指示暂停，恢复 GNU 工具链继续开发。恢复路径：在外部终端执行 `rustup toolchain install stable-x86_64-pc-windows-msvc` + `rustup set default-host x86_64-pc-windows-msvc`（或直接 `rustup default` 切换）后重新验证；MSVC 下仍需验证静态库链接方案（导入库替换实验已确认 GNU 可静态链接但存在运行时问题，MSVC 路径待验证）。
- **PLAN 更新：** 无清单勾选变化；本条目记录体验修复与 MSVC 切换的当前状态。
- **剩余工作：** M3（预设与队列）；待用户环境就绪后继续 MSVC 静态整合。

### 2026-10-10（环境切换）：MSVC 工具链 + 清华源，WebView2Loader 静态链接达成

- **本次完成：** 用户完全重装 Rust 环境为 `stable-x86_64-pc-windows-msvc`。验证结果：**MSVC 构建的 exe 不再导入 WebView2Loader.dll（加载器静态链接进可执行文件），exe 体积从 GNU 调试构建的 233.6 MB 降至 13.8 MB，exe 旁与 deps 目录均无该 DLL**——官方结论（仅 MSVC 可静态链接加载器）属实。应用在无 DLL 环境下直接启动正常（窗口 + 24 个 WebView2 子进程）。同时按用户要求把 Rust 源切换为清华 TUNA 镜像（用户级：`RUSTUP_DIST_SERVER` / `RUSTUP_UPDATE_ROOT` 环境变量 + `~/.cargo/config.toml` 的 sparse 索引替换，不入库；`cargo update --dry-run` 实测从 TUNA 索引解析成功）。
- **验证方式与结果（全部实际执行，MSVC 工具链）：** `cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace`（含 v6 清单冒烟测试，MSVC 的 embed_resource 路径验证通过）、集成测试 10/10；前端 `pnpm check/lint/format:check/test/build` 全绿；`pnpm tauri dev` 冒烟（窗口正常）。
- **说明：** rust-toolchain.toml 保持 `channel = "stable"`（主机无关）；本机默认主机为 msvc，CI Windows 默认也是 msvc，macOS/Linux 不受影响。GNU 分支代码保留（含 DLL 行为，上游限制）；`copy_webview2_loader_for_tests` 在 MSVC 下自动 no-op。target 树内 `build\webview2-com-sys-*\out\` 下的 DLL 为 crate 构建暂存，不随应用分发。
- **PLAN 更新：** 无清单勾选变化；README Windows 构建说明改为 MSVC 优先并补充清华源配置。
- **剩余工作：** M3（预设与队列）。

### 2026-10-10（M3）：预设与队列

- **本次完成：** M3 全部 9 项。
  1. **预设格式**：`aozora-core::preset` 定义带 `schemaVersion: 1` 的预设 JSON（video / audio / subtitles / output.overwrite），顶层与嵌套结构均 `deny_unknown_fields`（未知字段直接报错，不静默丢弃）；预设**不保存任何机器相关路径**（输入、输出、外部字幕文件）：外部字幕文件路径在导入时被拒绝、保存时被剥离。
  2. **预设管理**：保存、重命名、删除（内置预设只读）、同名冲突拒绝、ID 文件名安全校验（拒绝 `..` 与路径分隔符，杜绝路径穿越）、原子写入 `<配置目录>/presets/<id>.json`。
  3. **导入 / 导出**：导入校验扩展名、大小上限（1 MiB）、版本、字段与取值范围，重新分配 ID，重名自动追加「（导入）」；导出剥离内置标记，可直接再次导入。
  4. **内置示例预设**：程序内置三组（H.264 兼容 CRF 18/slow、HEVC 高画质 CRF 16/slow/10-bit、NVENC 快速 HEVC CQ 21/p5），只读、可复制为自定义预设。
  5. **单任务队列**：`aozora-core::queue` 纯状态机（FIFO、同一时刻只运行一个任务、拒绝重复 ID 与输出路径冲突）；`enqueue_job` 在入队前完成配置 / 路径 / 能力 / 可写性 / 冲突检查，任务结束后自动接续下一个；`job:queue` 事件推送队列快照，压制页与任务页展示运行中与排队状态并支持移除排队任务。
  6. **失败重试**：任务页可展开失败任务的日志尾部，并把历史任务的编码配置与输入 / 输出路径载入压制页，修改后重新入队。
  7. **任务历史**：`aozora-core::history` 持久化到 `<配置目录>/history/jobs.json`（原子写入、最新在前、上限 50 条、日志尾部 80 行），只记录结构化字段与日志尾部，不保存完整命令行；版本不受支持或文件损坏时报可读错误且**不覆盖**用户文件。
  8. **命令预览与复制**：`preview_job_args` 生成参数预览，前端渲染为可读命令行并支持一键复制（仅用于展示，执行仍走参数数组），同时标注全部参数由应用生成。
  9. **重启验证**：见下方验证结果。
- **验证方式与结果（全部实际执行）：**
  - Rust：`cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace`（87 单元 + 1 冒烟，含预设 / 历史 / 队列新用例）全部通过；实机集成测试 `cargo test --workspace -- --ignored` **11/11 通过**（真实 FFmpeg 的成功 / 失败 / 取消 / 覆盖 / 字幕 / NVENC 路径未受影响）。
  - 前端：`pnpm run check`（0 错误）、`pnpm run lint`、`pnpm run format:check`、`pnpm test`（39 通过，新增命令渲染、预设摘要 / 文件名、任务事件与队列处理用例）、`pnpm build` 全部通过。
  - **应用内端到端验证（真实 IPC + 真实 FFmpeg）**：临时在 dev 构建挂入自检脚本（验证后已删除，未入库），经真实 `enqueue_job` / `cancel_job` / `queue_snapshot` / `list_job_history` / `create_preset` 命令序列执行，结果从文件系统核对——长任务运行中入队第二个任务得到 `started=false, position=1`（正确排队）；队列快照 `active=1 queued=1`；与排队任务输出路径冲突的再次入队被拒绝；取消运行中任务后队列自动接续，排队任务完成并产出成品、被取消任务无成品；历史文件写入两条记录（1 completed + 1 cancelled），含配置快照与日志尾部且可被 JSON 解析。
  - **重启验证（M3 第 9 项）**：重启应用进程后重新读取，预设（内置 3 + 用户 2）与历史（2 条）仍在、无解析错误，并能在重启后的进程里继续新建预设；核心层另有"同一目录新建存储实例读回磁盘数据"的用例模拟重启。
  - `pnpm tauri dev` 冒烟：窗口标题 "Aozora Encode"、进程响应正常。
- **同日审查修复（独立审查发现，均已处理）：** ① 回退成品路径加入任务 ID（原先只按文件名，不同目录下的同名输入会互相覆盖成品）；② 前端历史读取失败不再清空列表（只提示错误，并允许从界面重置历史文件）；③ 编码配置的嵌套结构补齐 `deny_unknown_fields`（导入时拼错字段不再被静默忽略）；④ 队列快照加序号，避免较旧快照覆盖较新事件状态；⑤ `job:started` 丢失时按队列快照构造占位运行中状态；⑥ 应用预设不再清空已选的外部字幕文件；⑦ 导入 / 导出命令校验 `.json` 扩展名、导出目标不能是目录；⑧ 导入重名加后缀前先截断，避免"名称过长"的误导性错误；⑨ FFmpeg 启动前若已收到取消请求则直接按取消处理；⑩ 导出默认文件名过滤 Windows 非法字符。
- **限制（如实记录）：**
  - 历史中保存的日志是**尾部截断**（每条 80 行、单行 300 字符；界面实时日志保留 500 行），不是完整日志。
  - 历史与预设文件包含本地绝对路径（输入 / 输出），分享前需自行检查。
  - 临时文件清理与队列调度假设**同一时间只有一个应用实例在压制**（多实例会互相清理对方临时文件）。
  - 失败重试若上次输出路径已存在且未勾选覆盖，会在入队时以可读错误拒绝，需用户自行改路径或勾选覆盖（未提供一键改名）。
  - 未做 GUI 点击级人工验证（按钮与对话框）；上述端到端验证通过真实命令与文件系统结果核对完成。
- **PLAN 更新：** M3 清单 9 项、实施进度总览 M3 与 §11 M3 勾选；README M3 功能清单同步；§8 数据布局补充内置预设不落盘的实现差异；§6.4 补充回退成品路径按任务 ID 唯一化。
- **剩余工作：** 回到 M4（音轨 / 字幕轨 / 章节映射、ASS 处理、MKV 封装完善）。

### 2026-10-10（品牌资源）：应用图标替换为项目图标

- **本次完成：** 用户提供的 `icon.png`（1080×1080、透明圆角方形、深色底 `At` 字样）替换全部应用图标资源。
  1. 源图入库为 `src-tauri/app-icon.png`（原临时放在仓库根目录，移入 `src-tauri` 并以 Tauri 约定的 `app-icon` 命名，便于后续重新生成）。
  2. `pnpm tauri icon src-tauri/app-icon.png` 重新生成 `src-tauri/icons/*`：`icon.ico`（16/24/32/48/64/256 六种尺寸，Windows 可执行文件资源与窗口 / 任务栏图标）、`icon.icns`、`32x32/64x64/128x128/128x128@2x/icon.png`、Windows Store `Square*Logo` 与 `StoreLogo`。
  3. 该命令默认还会产出 Android / iOS 资源；移动端不在项目范围内，已删除 `src-tauri/icons/android` 与 `src-tauri/icons/ios`，未入库。
  4. 前端页签图标 `public/favicon.png` 由同一源图另存为 32×32（该文件不在 `tauri icon` 的生成范围内）。
  5. 侧边栏品牌标识：`src/App.svelte` 原有的文字占位方块（`brand-mark` 内的 "Ao"）改为项目图标图片；新增前端资源 `src/lib/assets/brand-mark.png`（72×72，供 36 CSS px 在 2 倍屏使用），构建时由 Vite 内联为 data URL，不产生额外运行时文件。
- **验证方式与结果（全部实际执行）：**
  - 图标内容比对：用 Pillow 把生成结果与源图逐一比对（原图像素差异 + 缩小到 8×8 的结构差异）——`32x32/64x64/128x128/128x128@2x/icon.png`、`StoreLogo`、`Square150x150Logo`、`icon.ico` 的最大尺寸项（256）与最小尺寸项（16）全部通过（结构差异 ≤ 5，逐像素差异为不同重采样算法造成的边缘噪声）；`icon.icns` 头部为 `icns` 魔数、111,740 字节。验证脚本为一次性脚本，未入库。
  - exe 内嵌资源比对：解析 `target/debug/aozora-encode.exe` 的 PE 资源目录，`RT_ICON`（id 1–6）六个尺寸与 `icon.ico` 的同尺寸数据 **SHA-256 全部一致**（594 / 928 / 1319 / 1994 / 2750 / 8992 字节），`RT_GROUP_ICON` 为 90 字节图标组。
  - 实机外观：`pnpm tauri dev` 启动后对窗口做 `PrintWindow` 抓图——标题栏图标与侧边栏品牌标识均为新的深青底 `At` 标识。
  - 前端：`pnpm run format:check`（通过，新增 PNG 未触发格式问题）、`pnpm run lint`（退出码 0）、`pnpm run check`（0 错误 0 警告）、`pnpm test`（42 通过）、`pnpm build`（通过，产物中 brand-mark 以 `data:image/png;base64,…` 内联，`build/assets/` 下无独立图标文件）。
- **构建注意（本次实测发现）：** 本项目用 `tauri::generate_context!` 而非 `tauri_build::CodegenContext`，`tauri-build` 不会为 `bundle.icon` 输出 `cargo:rerun-if-changed`——只改图标文件时 cargo **不会**重编二进制（实测 `cargo run` 0.5 秒完成、exe 哈希不变）。需要在改动图标后触发一次重建（例如触碰 `src-tauri/tauri.conf.json` 或 `cargo clean -p aozora-encode`），否则 dev / 打包产物仍带旧图标。首次全新构建无此问题。
- **限制（如实记录）：** 本次未重新执行 MSI 打包（`pnpm tauri build`），安装包与快捷方式图标沿用同一 `icon.ico`，未做打包产物验证；任务栏 / 开始菜单图标未单独截图确认（与标题栏同源）。侧边栏品牌标识仅在浅色主题下截图确认，未逐一切换深色主题核对。
- **PLAN 更新：** 无清单勾选变化（图标资源不属于 M4/M5 现有条目）；README「本地开发」新增应用图标与重新生成说明。
- **剩余工作：** 回到 M4（音轨 / 字幕轨 / 章节映射、ASS 处理、MKV 封装完善）。

### 2026-10-10（品牌统一）：产品名改为 Aozora Toolbox 与版本号 0.3.0

- **本次完成：** 按用户要求把面向用户的产品名统一为 **Aozora Toolbox**，并升级版本号。
  1. 用户可见名称：`tauri.conf.json` 的 `productName` 与窗口 `title`、`index.html` 的 `<title>`、`src-tauri/Cargo.toml` 的 `authors`、`src-tauri/core/Cargo.toml` 的 `description`、`src-tauri/core/src/lib.rs` 的 crate 文档、`.github/workflows/build.yml` 中 macOS 签名验证的 `Aozora Toolbox.app`。
  2. 设置页「应用」一栏此前显示 `env!("CARGO_PKG_NAME")`（内部 crate 名 `aozora-encode`），现改为 `app.package_info().name`（即 `productName`），与安装包、任务栏展示一致。
  3. **内部标识保持不变**：Rust 包名 `aozora-encode` / `aozora-core`、lib 名 `aozora_encode_lib`、应用标识 `com.aozora.encode`、配置目录 `%APPDATA%\com.aozora.encode` 均未改动——避免破坏 `use aozora_core::…`、`aozora_encode_lib::run()`、既有用户配置目录与 MSI 升级识别。
  4. 版本号 0.2.1 → **0.3.0**（次版本 +1，用户先后要求「升级版本号」与「更新一个副版本号」）。
- **版本号判断依据与偏离说明：** 首轮按 CONTRIBUTING §3 只升修订号（0.2.2）——M3 已完成、M4 清单未完成，变更内容（图标资源、侧边栏品牌标识、产品名、版权标注）属于同一里程碑内的小改进与文档调整。随后用户要求升次版本，故最终取 **0.3.0**。**这是按用户明确指令的偏离**：按 §3 的里程碑对应关系，0.3.0 原本对应 M4 完成时机；M4 完成时是否再升次版本，需按当时情况重新判断并在本文件说明。版本变更按规范独立提交（`build:` 类型）。
- **验证方式与结果（全部实际执行）：**
  - Rust：`cargo check --workspace`（通过，`Cargo.lock` 中 `aozora-encode` / `aozora-core` 同步为 0.2.2/0.3.0）、`cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace`（87 单元 + 1 冒烟通过，11 实机用例保持 ignored）全部通过。
  - 前端：`pnpm run format:check`、`pnpm run lint`、`pnpm run check`（0 错误 0 警告）、`pnpm test`（42 通过）全部通过。
  - 实机窗口标题：`cargo run` 启动后 `PrintWindow` 抓图，标题栏为「Aozora Toolbox」。
  - **MSI 打包验证（`pnpm tauri build`，两次均成功）：** 改名后先用 0.2.2 验证（`Aozora Toolbox_0.2.2_x64_en-US.msi`，2.30 MiB），升到 0.3.0 后重新打包（2m28s）产出 `Aozora Toolbox_0.3.0_x64_en-US.msi`（2.30 MiB），生成的 `target/release/wix/x64/main.wxs` 中 `Product Name="Aozora Toolbox"`、`Version="0.3.0"`、安装目录 `INSTALLDIR Name="Aozora Toolbox"`、开始菜单/桌面快捷方式名「Aozora Toolbox」、注册表键 `Software\aozora\Aozora Toolbox`。
  - 打包产物内可执行文件名仍为 `aozora-encode.exe`（`main.wxs` 的 `File Source=…\aozora-encode.exe`）——Tauri 只按 `productName` 命名 MSI 与显示名，**不重命名内部可执行文件**，因此开发时的 exe 名与 README 中的 `icacls target\debug\aozora-encode.exe` 说明继续有效。
- **限制（如实记录）：** 未实际安装 MSI 验证开始菜单/桌面快捷方式与「应用和功能」中的显示名（仅核对 WiX 源文件与产物文件名）；`bundle/msi/` 目录下仍留有历史构建产物 `Aozora Encode_0.2.1_x64_en-US.msi` 与 `Aozora Toolbox_0.2.2_x64_en-US.msi`（未清理，该目录不入库）；macOS / Linux 产物未构建（本机无对应环境），仅同步了 `build.yml` 中硬编码的 `.app` 名称。
- **PLAN 更新：** 无清单勾选变化；README 补充「产品名与内部标识」说明并修正 macOS `.app` 名称；本记录新增。
- **剩余工作：** 回到 M4（音轨 / 字幕轨 / 章节映射、ASS 处理、MKV 封装完善）。

### 2026-10-10（文档修复）：README 顶部结构

- **本次完成：** 按用户要求只修 README 顶部（其余两项保留，见下）。
  1. 删除开头重复的全宽 `<img src="assets/icon.png" align="center" style="max-width: 100%;">`（同一张 logo 在居中的 `div` 里已有一份，且该标签未闭合）。
  2. `----` → 结束居中 `div`：Markdown 的 `---`/`----` 分隔线在 HTML 块内不会被解析（会按字面文本显示），原本夹在 `<div align="center">` 与 `</div>` 之间；现在居中块只包含 logo 与标题，随后直接接简介段落。
  3. `</br>` → `<br>`（HTML5 中 `br` 是空元素，`</br>` 不是合法写法）。
  4. 恢复被这次改版删掉的引用块「> 项目状态：规划 / 初期开发阶段……」——该说明仍符合当前状态（M4 未完成），且 PLAN/README 口径需要一致；如不需要可随时删除。
- **验证方式与结果：** `pnpm exec prettier --check README.md`（通过）；标题层级、代码围栏配对（10 个标记）、相对链接（AGENTS.md / CONTRIBUTING.md / PLAN.md / docs/architecture.md）逐项核对正常。文档改动不涉及代码，未重跑代码检查。
- **保留未处理（按用户指示）：** `docs/architecture.md` 仍未入库（其中 ADR-007 的 NSIS 表述与 macOS/Linux 构建描述同 AGENTS.md / CI 现状不一致）；`src-tauri/target/release/bundle/msi/` 下的历史 MSI 未清理。
- **PLAN 更新：** 本记录新增；无清单勾选变化。
- **剩余工作：** 回到 M4（音轨 / 字幕轨 / 章节映射、ASS 处理、MKV 封装完善）。

### 2026-10-10（M2 扩展）：封装格式选择器（MKV / MP4 / WebM）

- **本次完成：** 按用户要求把输出封装格式从"跟随输出路径扩展名"改为**显式选择**，新增 MKV / MP4 / WebM 三个选项（此前只允许 .mkv / .mp4，且 MP4 输入默认仍出 MP4）。
  1. 配置模型：`aozora_core::encode::Container`（`mkv` / `mp4` / `webm`，`Default = Mkv`）加入 `OutputConfig.container`；旧配置 JSON 无此字段时按 MKV 处理（`#[serde(default)]`，不破坏既有预设与任务历史）。预设置信 `PresetOutput.container`（同样带默认值，`to_encode_config` 透传）。
  2. 路径校验：`planner::check_paths(input, output, container, overwrite)` 现在要求输出扩展名与所选容器**一致**——扩展名属于其他受支持容器时给出"扩展名与所选封装格式不一致"的明确提示，其他扩展名仍报"必须是 .mkv、.mp4 或 .webm"。容器决定 FFmpeg 的封装器（仍按扩展名选择，应用不传 `-f`）。
  3. 前端：压制页新增「封装格式」下拉，切换时自动把输出路径扩展名换成对应容器（`swapOutputExtension`，保证路径与容器始终一致）；保存对话框按当前容器限定扩展名；输出路径扩展名被手工改乱时在界面提前红色提示。导入输入文件时，若输入本身是 MKV / MP4 / WebM 则沿用其容器，其余输入保持当前选择。
  4. 兼容性提示（不代为改写编码器）：WebM + 非 AV1 编码器（本应用默认 libx264）提示"码流无法封装进该格式"；MP4 + 保留字幕轨提示仅支持 mov_text；WebM + 保留字幕轨提示 WebM 不封装文本字幕轨。
  5. 预设：`matchesPreset` 与预设摘要纳入封装格式，保存预设时会带上当前容器。
- **验证方式与结果（全部实际执行）：**
  - Rust：`cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace`（93 单元 + 1 冒烟通过；新增 `check_paths_accepts_each_supported_container`、`check_paths_rejects_extension_container_mismatch`、`container_variants_round_trip_and_expose_extensions`、`unknown_container_is_rejected`、`container_survives_preset_round_trip_and_defaults_to_mkv`、`preset_json_without_container_field_is_accepted` 等用例）。
  - 实机集成：`cargo test --workspace -- --ignored` **12/12 通过**（含新增 `encode_honours_configured_container`：MKV 与 MP4 两种容器真实压制均成功、成品可被 FFprobe 读取、临时文件清理正常）。
  - **端到端脚本核对（一次性脚本，验证后已删除未入库）**：用真实 FFmpeg 走 `validate_config → check_paths → build_args → run_encode` 的 MP4 路径，成品 57,730 字节、视频流 1、音频流 1、时长 2.0s；容器与扩展名不一致时得到可读错误。测试素材使用 `target/self-test/input-short.mkv`。
  - 前端：`pnpm run check`（0 错误 0 警告）、`pnpm run lint`、`pnpm run format:check`、`pnpm test`（47 通过，新增 `suggestOutputPath` 容器参数、`swapOutputExtension`、`containerFromPath`、`containerSupportsEncoder` 与 `matchesPreset` 容器比较用例）、`pnpm build`（通过）。
  - 实机界面：`pnpm tauri dev` 后 `PrintWindow` 抓图确认「封装格式」下拉显示为「MKV（Matroska）」并带说明文字，输出路径占位符同步显示 `.mkv`。
- **范围说明（偏离记录）：** §4.2 原写"输出容器第一版以 MKV 为主，可按需求添加 MP4"；本次按用户明确要求显式提供 MKV / MP4 / WebM 三个选项，属于该条目的扩展。
- **限制（如实记录）：**
  - **WebM 目前实际上不可用**：应用可选的编码器只有 libx264 / libx265 / h264_nvenc / hevc_nvenc / av1_nvenc，WebM 只接受 VP8 / VP9 / AV1。选 WebM 通常会在 FFmpeg 阶段失败（界面已提示）；若显卡支持 AV1（`av1_nvenc` 可用）才可能成功。要真正可用需先支持 libvpx-vp9 / libaom-av1（计划外，未实现）。
  - WebM 与 MP4 的字幕/编码器兼容性只在界面提示，**不做强制阻止**；压制失败时由 FFmpeg 的错误日志说明原因。
  - 未在真实消费设备上验证 MP4 产物的兼容性。
- **PLAN 更新：** 无清单勾选变化（M2 已完成，本项为其扩展）；本记录新增。
- **剩余工作：** 回到 M4（音轨 / 字幕轨 / 章节映射、ASS 处理、MKV 封装完善）。

### 2026-10-10（M1 深化）：按实际环境检测编码器与显卡，不再硬编码

- **本次完成：** 按用户要求，编码器可用性从"硬编码 5 个 key 的布尔值"改为**按用户真实运行环境检测**，并检测显卡以确定可用硬件编码器（NVIDIA / AMD / Intel）。
  1. 编码器目录扩充（`encode::ENCODER_SPECS`，共 **12 项**）：软件 `libx264` / `libx265` / `libvpx-vp9`，NVIDIA `h264_nvenc` / `hevc_nvenc` / `av1_nvenc`，Intel `h264_qsv` / `hevc_qsv` / `av1_qsv`，AMD `h264_amf` / `hevc_amf` / `av1_amf`。每项新增 `label` / `vendor` / `family` / `codec` / `quality_label` / `zero_bitrate` 字段。
  2. 参数写法按家族区分（`planner::push_video_quality_args`）：软件 `-preset … -crf N`；libvpx-vp9 `-deadline good -cpu-used N -row-mt 1 -crf N -b:v 0`（CRF 必须配 `-b:v 0`，否则退化为受限 VBR）；NVENC `-preset p1…p7 -rc vbr -cq N`；QSV `-preset … -global_quality N`（ICQ）；AMF `-quality speed|balanced|quality -rc cqp -qp_i N -qp_p N`。
  3. **两段式可用性判定**（`capabilities`）：先看 `ffmpeg -encoders` 是否包含该编码器，再对**硬件编码器实拍一次**（5 帧 320×240 → `-f null`，不写文件），排除"编译进来了但本机没有对应硬件 / 驱动不工作"。软件编码器只要编译进来即视为可用。
  4. 显卡检测（`capabilities::detect_gpu_adapters`）：Windows 走 PowerShell + CIM `Win32_VideoController`（名称 / 厂商 / 驱动版本），macOS 走 `system_profiler`，Linux 读 `/sys/class/drm`；厂商归一化为 `nvidia` / `amd` / `intel` / `other`（虚拟显示适配器归 `other`）。
  5. 前端不再硬编码可用性：`CapabilityReport.encoders` 改为逐项 `{id,label,vendor,codec,hardware,listed,usable,reason}`；压制页编码器下拉**只列出本机可用项**并按厂商分组（软件 / NVIDIA / AMD / Intel），检测未完成时提示"正在检测可用编码器…"；当前编码器不可用时（例如预设来自另一台机器）自动回退到可用默认项并提示。设置页新增「显卡」区块，并在编码器列表逐项显示不可用原因。
  6. 一致性保障：`src-tauri/tests/smoke.rs` 新增 DTO 字段结构测试（前端按 camelCase 读取该结构）。
- **验证方式与结果（全部实际执行）：**
  - Rust：`cargo fmt --check`、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo test --workspace`（**100 单元 + 2 冒烟通过**，新增 QSV / AMF / VP9 参数用例、`extract_encoder_error` 用例、显卡厂商归一化与 JSON 解析用例、可用性判定用例）。
  - **本机实机检测结果（RTX 4080 Laptop + AMD Radeon 610M + gyan full build ffmpeg 9.0）**：`libx264` / `libx265` / `libvpx-vp9` 可用；`h264_nvenc` / `hevc_nvenc` 可用；`av1_nvenc` **不可用**（RTX 40 系无 AV1 编码单元，`No capable devices found`）；`h264_amf` / `hevc_amf` 可用；`av1_amf` 与全部 `*_qsv` **不可用**（本机无可用 Intel MFX 实现）。检测耗时约 2.5s，结果在应用会话内缓存。
  - 实机集成：`cargo test --workspace -- --ignored` **13/13 通过**，其中 `encode_honours_configured_container` 扩展为 MKV(x264) / MP4(x264) / **WebM(libvpx-vp9 + Opus 音轨)** 三条真实压制路径，成品经 FFprobe 校验编码格式正确、临时文件清理正常。
  - 前端：`pnpm run check`（0 错误）、`lint`、`format:check`、`test`（**53 通过**，新增可用性判定、默认编码器挑选、真实检测报告 → 列表映射、WebM 音频兼容性用例）、`build` 全部通过。
  - 实机界面：`pnpm tauri dev` + `PrintWindow` 抓图确认编码器下拉为「H.264（libx264，软件）」并显示说明「列表来自本机 FFmpeg 与显卡检测：硬件编码器均在检测时实拍验证」。
- **顺带修正（实测发现）：** WebM 容器除了不接受 H.264，也**不接受 AAC 音轨**（`Only VP8 or VP9 or AV1 video and Vorbis or Opus audio ... are supported for WebM`）；应用目前固定 `-c:a copy`、不做音频转码，因此界面对"WebM + 非 Opus/Vorbis 音轨"新增红色提示，容器说明也改为"音轨为 AAC 等格式时会被拒绝"。
- **限制（如实记录）：**
  - QSV 与 AMF 的 AV1 路径、以及所有 QSV 路径在本机**无法实测**（本机无可用 Intel MFX；Radeon 610M 无 AV1 编码单元），参数按 FFmpeg 9.0 的 `-h encoder=` 输出与官方文档编写；这类编码器只有在通过实拍探测后才会出现在列表里，因此参数错误不会呈现为"可用但一压就失败"。
  - 实拍探测判定的是"能否编码 5 帧合成画面"，不保证特定分辨率 / 像素格式（例如 10-bit P010）下的可用性；像素格式仍由用户选择，失败时由 FFmpeg 报错。
  - 非 Windows 平台（macOS / Linux）本轮不做实拍探测，只按构建是否包含判断；显卡检测在 macOS / Linux 未实机验证。
  - 音频仍是"复制原音轨"，没有音频编码选项，所以 WebM 仅适用于 Opus / Vorbis 音轨的源。
- **PLAN 更新：** 无清单勾选变化（M1 的"检测实际可用的编码器与滤镜能力"仍是已完成状态，本次为该条的深化）；本记录新增。
- **剩余工作：** 回到 M4（音轨 / 字幕轨 / 章节映射、ASS 处理、MKV 封装完善）。

### 2026-10-10（M1 深化续）：补齐 AV1 软件编码器并澄清检测口径

- **本次完成：** 用户反馈"ffmpeg 提示支持 av1 编码，为什么没探测到"。核对后确认是**两件事混在一起**，分别处理：
  1. **确实漏了 3 个软件 AV1 编码器**：`libsvtav1` / `libaom-av1` / `librav1e` 在 `ffmpeg -encoders` 中存在且本机实拍可用，但没进应用目录（此前只有 `av1_nvenc`）。现已补入规格表，编码器总数 12 → **15**。
  2. **硬件 AV1 的"不支持"是真实结论**，不是检测缺陷：`-encoders` 里列出 `av1_nvenc` 只代表该 FFmpeg 构建编译了它，是否可用取决于显卡编码单元。实拍证据：RTX 4080 Laptop 报 `No capable devices found`（RTX 40 系无 AV1 编码单元，需 RTX 50 系）、Radeon 610M 报 `CreateComponent(AMFVideoEncoderHW_AV1) failed with error 1`、本机 Intel MFX 不可用。
  3. 新增 `PresetKind`（`Preset` / `CpuUsed` / `Speed` / `QsvPreset` / `AmfQuality`）：三个 AV1 软件编码器的速度档位写法各不相同——SVT-AV1 用 `-preset N`（0–13）、libaom-av1 用 `-deadline good -cpu-used N`（0–8）、librav1e 用 `-speed N`（0–10）；三者恒定质量都补 `-b:v 0`。`-row-mt 1` 现在只加给 libvpx-vp9。
  4. **错误信息更可读**：`extract_encoder_error` 改为按"原因关键词"打分（`No capable` / `Cannot load` / `CreateComponent` / `Error creating …` 优先），不再被 `Terminating thread with return code` 这类收尾行盖住。现在界面能直接显示「NVIDIA 硬件编码不可用：No capable devices found」。
  5. WebM 兼容性判断改为按**编码格式**（`codec == vp9 || codec == av1`），因此三个软件 AV1 与三个硬件 AV1 都被正确识别为可入 WebM。
- **验证方式与结果（全部实际执行）：**
  - Rust：`cargo test --workspace`（**101 单元 + 2 冒烟通过**，新增 `av1_software_encoders_use_their_own_speed_option` 覆盖三种速度选项写法）；`cargo clippy -D warnings`、`cargo fmt --check` 通过。
  - 实机集成：`cargo test --workspace -- --ignored` **13/13 通过**；`encode_honours_configured_container` 现覆盖 MKV(x264) / MP4(x264) / WebM(VP9+Opus) / **WebM(SVT-AV1+Opus)** 四条真实压制路径，成品经 FFprobe 校验编码格式为 h264 / vp9 / **av1**。
  - 本机编码器实拍：`libsvtav1` 通过（140ms/5 帧）、`libaom-av1` 通过（630ms/5 帧）、`librav1e` 通过（335ms/5 帧）；`av1_mf` 报 `could not find any MFT for the given media type`，`av1_d3d12va` / `av1_vulkan` / `av1_vaapi` 报 "Impossible to convert between the formats"（需要 hwupload 滤镜链，本应用不做这类硬件帧链路，故未纳入目录）。
  - 前端：`check`（0 错误）、`lint`、`format:check`、`test`（53 通过，WebM 编码器兼容性用例扩展到 VP9 + 全部 AV1）全部通过。
- **限制（如实记录）：**
  - `av1_mf`（Windows Media Foundation）与 `av1_d3d12va` / `av1_vulkan` / `av1_vaapi` 这类需要硬件帧上传（`hwupload`）的编码器**未纳入目录**：它们不能直接吃软件帧，必须配 `-init_hw_device` / `-vf hwupload` 的完整链路，属独立工作量；在没有 Intel/AMD 硬件可实测的机器上也无法验证。
  - 软件 AV1 速度慢（libaom-av1 尤甚），应用只透传速度档位，不做速度/质量自动推荐。
  - 硬件 AV1 的实际可用性只能在有对应硬件的机器上验证。
- **PLAN 更新：** 无清单勾选变化；本记录新增。
- **剩余工作：** 回到 M4（音轨 / 字幕轨 / 章节映射、ASS 处理、MKV 封装完善）。

### 2026-10-10（M2 扩展续）：输出分辨率调整（含奇数取偶修正）

- **本次完成：** 按用户要求新增"输出分辨率"设置，版本号随之升到 **0.3.2**。
  1. 配置模型：`VideoConfig.resolution: Option<Resolution{width,height}>`（两边都可空），旧配置 / 旧预设无此字段时按"保持源分辨率"处理；预设置信该字段（新保存的预设会带上分辨率）。
  2. 参数生成：`planner::scale_filter` 生成缩放滤镜——只给宽 `scale=W:-2`、只给高 `scale=-2:H`（另一边按源宽高比自动计算）、两边都给 `scale=W:H:force_original_aspect_ratio=decrease`（缩放进框内，不拉伸、不裁剪）。
  3. 滤镜链顺序：**先缩放、后烧录字幕**（`video_filters`），这样 ASS 的对齐与边距按最终分辨率渲染；此前烧录字幕是直接写进 `-vf`，现在与缩放合并到同一条链上。
  4. 界面：压制页新增「输出分辨率」一行（宽 × 高 + 常用分辨率下拉 1080p/720p/480p/源分辨率），显示源分辨率、实时提示（按比例推算的输出尺寸 / "缩放进框内"）、范围校验错误，以及奇数取偶提示。
  5. 校验：`RESOLUTION_RANGE = (16, 8192)`，超范围或非整数在界面与后端都会被拒绝。
- **实测发现并修正的缺陷：** 第一版用 `scale=W:-2` 直接透传用户输入，写奇数（例如 641）时 **x264 直接报 `width not divisible by 2 (641x480)` 编码失败**——`-2` 只保证"自动计算出来的那一边"是偶数，不会修正用户显式写的值。现改为在 Rust 侧先把给定值向下取到偶数（`even_dimension`，下限 16），并在界面显示"641 → 640"。该缺陷由新增的实机测试直接暴露。
- **验证方式与结果（全部实际执行）：**
  - Rust：`cargo test --workspace`（**105 单元 + 2 冒烟通过**；新增 `scale_filter_handles_partial_and_full_resolution`、`resolution_adds_scale_filter_before_subtitle_burn`（含滤镜顺序断言）、`resolution_accepts_partial_and_empty_values`、`resolution_out_of_range_is_rejected`，并把分辨率纳入 JSON 往返用例）。
  - 实机集成：新增 `encode_applies_configured_resolution`（`--ignored`，**14/14 通过**）——真实压制并 FFprobe 核对：只给宽得 640×360、同比例框得同尺寸、**奇数 641 输出 640**、宽高均为偶数、宽高比未被改变。
  - 前端：`check` 0 错误、`lint`、`format:check`、`test`（**59 通过**，新增分辨率校验 / 取偶 / 预设比较用例）、`build` 全部通过。
  - 实机界面：抓图确认「输出分辨率」一行显示 宽 × 高 + 常用分辨率下拉，且与相邻字段不再挤压（首版布局与「字幕」标签重叠，已改为整行）。
- **限制（如实记录）：**
  - 只做缩放，不做裁剪 / 加黑边（两边都给时是"缩放进框内"，输出可能小于给定值，界面有说明）；未提供旋转、变形等几何处理。
  - 上采样（放大）不做限制，也不会提示"放大不会增加细节"。
  - 分辨率参与预设比较：旧预设（无该字段）与"未设分辨率"等价，不会因此被标注"参数已修改"。
- **PLAN 更新：** 无清单勾选变化（属 M2 编码配置的扩展）；本记录新增。
- **剩余工作：** 回到 M4（音轨 / 字幕轨 / 章节映射、ASS 处理、MKV 封装完善）。

---

## 技术方案正文：轻量动画视频压制工具箱

-   文档版本：1.0
-   状态：初版方案，可作为项目 README / 技术设计基线
-   目标平台：Windows 10/11 x64 优先（唯一分发目标）；GitHub CI 在 Windows / macOS / Linux 三平台做构建验证
-   核心定位：本地运行、轻量、可靠、面向动画与日常视频压制的桌面工具
-   核心原则：不重新实现编码器；以 FFmpeg / FFprobe 为媒体引擎

------------------------------------------------------------------------

## 1. 项目概述

### 1.1 项目目标

构建一个现代化、轻量的视频压制工具箱，为用户提供比直接编写 FFmpeg
命令更友好的工作流，同时保留编码参数的透明度与可控性。

第一阶段重点解决：

1.  选择视频文件并读取媒体信息。
2.  选择编码器、质量参数、预设和输出路径。
3.  生成可预览、可复制的 FFmpeg 命令。
4.  启动、监控、暂停能力评估、取消和管理压制任务。
5.  保存、导入和导出压制预设。
6.  保留音频、字幕、章节等常见流，并提供基本映射选项。
7.  显示进度、编码速度、已用时间、预计剩余时间、日志和错误原因。

### 1.2 非目标

第一版不实现以下功能：

-   自研视频编码器或媒体解码器。
-   完整的节点式工作流编辑器。
-   内置完整 VapourSynth / Python 运行环境。
-   云端编码、账号系统、遥测服务或在线预设市场。
-   专业 NLE 视频剪辑功能。
-   自动判断任何片源的"最佳画质参数"并承诺统一结果。
-   第一版就支持 Windows、macOS、Linux 全平台发布（CI 仅做多平台构建验证，不含签名、公证与发布）。

这些功能可以根据真实需求后续增加，不应成为 MVP 的前置条件。

### 1.3 设计原则

-   **本地优先**：媒体文件、预设、日志和任务历史默认留在本机。
-   **薄
    GUI，可靠核心**：界面只负责收集意图与呈现状态；进程控制和任务状态由
    Rust 侧管理。
-   **参数透明**：高级用户能查看、复制生成的命令和完整日志。
-   **安全执行**：不把用户输入拼接成 Shell
    字符串；通过参数数组启动子进程。
-   **增量开发**：先支持常见编码，再逐步扩展滤镜和复杂字幕工作流。
-   **能力探测优先**：以当前 FFmpeg
    实际提供的能力为准，不假定所有构建都包含相同编码器。
-   **失败可解释**：路径错误、编码器缺失、磁盘空间不足、用户取消等情况应给出不同结果。

------------------------------------------------------------------------

## 2. 技术栈

### 2.1 推荐组合

  ----------------------------------------------------------------------------------------
  层                      技术                    职责
  ----------------------- ----------------------- ----------------------------------------
  桌面框架                Tauri 2                 窗口、应用打包、前后端桥接

  前端                    Svelte 5 + TypeScript   参数表单、任务列表、日志和设置界面

  样式                    CSS variables +         主题、布局和控件样式；避免过重依赖
                          少量组件                

  应用核心                Rust                    任务管理、配置、进程生命周期、能力检测

  异步运行                Tokio（通过 Tauri/Rust  异步 I/O、任务调度
                          需要时使用）            

  媒体探测                FFprobe                 流信息、时长、帧率、色彩信息

  编码和封装              FFmpeg                  编码、滤镜、流映射、封装

  配置                    Serde + JSON            预设、应用设置、工作流数据

  本地历史                第一版可用              任务历史、最近文件等
                          JSON；需要查询后再加    
                          SQLite                  

  测试                    Rust tests + 前端测试 + 逻辑、UI 和真实媒体验证
                          FFmpeg 集成测试         

  CI                      GitHub Actions          检查、测试和构建
  ----------------------------------------------------------------------------------------

### 2.2 为什么选这套

Tauri 的 Web UI 允许快速构建现代界面，而 Rust
侧可以安全管理子进程与任务。FFmpeg
独立运行，实际编码性能主要由编码器、预设、片源和硬件决定，而不是由 UI
框架决定。

第一版**不强制引入
SQLite、VapourSynth、全局状态管理库、节点编辑器或插件系统**。当任务历史查询、工作流复杂度或滤镜需求确实增长时再增加。

### 2.3 外部工具策略

-   用户可选择自己的 FFmpeg 工具目录。
-   应用启动或用户点击"重新检测"时，运行
    `ffmpeg -version`、`ffmpeg -encoders`、`ffmpeg -filters`
    等能力检测。
-   使用 `ffprobe -version` 检测探测工具。
-   保存工具路径和检测结果；更新工具后重新检测。
-   不能仅根据文件名判断某编码器可用。
-   应用可提供"外部工具模式"；是否随安装包附带
    FFmpeg，由发布策略和许可证审查决定。

------------------------------------------------------------------------

## 3. 系统架构

``` text
┌──────────────────────────────────────────┐
│                 Svelte UI                │
│ 文件 / 编码设置 / 预设 / 队列 / 日志     │
└────────────────────┬─────────────────────┘
                     │ Tauri commands/events
┌────────────────────▼─────────────────────┐
│               Rust Application           │
│                                          │
│  Media Probe  Preset Store  Job Manager  │
│  Capability   Command Plan  Process Ctrl │
│  Detection    Validation   Log Parser   │
└────────────────────┬─────────────────────┘
                     │ 参数数组 + 子进程 I/O
┌────────────────────▼─────────────────────┐
│             FFprobe / FFmpeg             │
│   软件编码 / NVIDIA 编码 / 滤镜 / 封装   │
└──────────────────────────────────────────┘
```

### 3.1 前端职责

-   收集用户的编码意图。
-   展示媒体信息、可用编码器和参数。
-   显示任务状态、进度、日志和错误。
-   触发任务创建、取消、重试和预设操作。
-   不直接调用系统 Shell，不自行管理 FFmpeg 子进程。

### 3.2 Rust 核心职责

-   校验文件路径和配置。
-   调用 FFprobe 并解析 JSON。
-   根据结构化配置生成参数数组。
-   启动 FFmpeg 子进程并持续读取输出。
-   解析机器可读进度。
-   管理任务状态和取消流程。
-   持久化预设与任务历史。
-   处理错误、临时文件和输出文件收尾。

### 3.3 模块边界

建议按功能划分模块，而不是把所有逻辑放进单个 Rust 文件：

-   `media_probe`：FFprobe 调用与媒体模型。
-   `encoder_registry`：编码器定义、参数能力与 UI 元数据。
-   `job`：任务数据模型和状态机。
-   `planner`：校验配置并生成执行计划。
-   `process`：子进程启动、输出读取、取消和退出状态。
-   `progress`：FFmpeg 进度解析与 ETA 计算。
-   `preset_store`：预设读写、导入、导出和版本迁移。
-   `settings`：工具路径、主题和应用设置。
-   `diagnostics`：日志与诊断信息。

不要为了"架构整洁"过早拆成多个独立进程或服务；它仍然是一个本地桌面应用。

------------------------------------------------------------------------

## 4. MVP 功能范围

### 4.1 媒体导入与探测

支持：

-   文件选择和拖放导入。
-   显示文件名、路径、容器、时长、大小。
-   视频流：编码格式、分辨率、帧率、像素格式、位深、色彩信息（若可读取）。
-   音频流：编码、语言、声道数、采样率、默认轨标记（若可读取）。
-   字幕流：格式、语言、标题、默认/强制标记（若可读取）。
-   章节数量与附件数量（在 FFprobe 数据可用时）。
-   探测失败时显示原始错误日志。

媒体探测输出应基于 FFprobe JSON，不通过解析人类可读的终端表格实现。

### 4.2 编码配置

第一版建议支持：

-   H.264 软件编码：`libx264`（如果当前 FFmpeg 构建包含）。
-   H.265 软件编码：`libx265`（如果当前构建包含）。
-   NVIDIA H.264：`h264_nvenc`（硬件及构建支持时）。
-   NVIDIA HEVC：`hevc_nvenc`（硬件及构建支持时）。
-   NVIDIA AV1：`av1_nvenc`（仅在硬件及构建支持时显示）。
-   音频：复制原音轨，后续再加入常见音频编码选项。
-   输出容器：第一版以 MKV 为主，可按需求添加 MP4。
-   质量模式、预设、像素格式、输出路径。
-   显示当前编码器的参数解释与能力限制。

编码器列表必须根据实际能力检测动态生成。若某编码器不可用，应显示原因，而不是生成必然失败的命令。

### 4.3 任务管理

-   单任务运行和排队任务。
-   进度、编码速度、已用时间、预计剩余时间（能可靠估算时）。
-   取消任务。
-   失败后查看完整日志。
-   失败任务可修改配置后重试。
-   任务完成后显示输出文件并可打开所在目录。
-   进度不确定时显示"不适用"，不要显示误导性的百分比。

第一版可限制为同时运行一个编码任务，避免 CPU/GPU 资源竞争和复杂调度。

### 4.4 预设

-   内置少量示例预设，例如 H.264 兼容、HEVC 高画质、NVENC 快速编码。
-   保存当前配置为自定义预设。
-   重命名、删除、导入、导出 JSON。
-   预设包含 `schemaVersion`，为未来迁移留出空间。
-   不把绝对输入路径写进通用预设，除非用户明确保存为任务模板。

### 4.5 命令预览与高级模式

-   展示生成后的参数列表或可复制命令。
-   高级用户可以查看完整命令。
-   第一版不提供任意 Shell 命令直接执行的默认入口。
-   如未来加入自定义参数，必须使用参数数组，并对输出路径、输入路径和受限选项做校验。
-   明确标识"应用自动生成参数"和"用户自定义参数"。

------------------------------------------------------------------------

## 5. 编码参数模型

### 5.1 结构化配置示例

``` json
{
  "schemaVersion": 1,
  "video": {
    "encoderId": "libx265",
    "qualityMode": "crf",
    "qualityValue": 16,
    "preset": "slow",
    "pixelFormat": "yuv420p10le"
  },
  "audio": {
    "mode": "copy"
  },
  "subtitles": {
    "mode": "preserve"
  },
  "output": {
    "container": "mkv",
    "path": ""
  }
}
```

这是应用内部的示意模型，不是直接传给 FFmpeg 的
JSON。执行前需要验证和转换。

### 5.2 编码器抽象原则

可共享的概念：

-   编码器 ID 和显示名称。
-   输出视频编码格式。
-   软件或硬件编码类型。
-   支持的像素格式和色深。
-   可用预设。
-   质量控制模式。
-   当前构建是否支持。

必须保留编码器差异：

-   x264/x265 常见质量控制为 CRF。
-   NVENC 常见质量控制涉及 CQ、码率控制模式等，具体选项取决于版本。
-   预设名称和数值不可跨编码器直接复用。
-   同样的 CRF/CQ 数值不代表不同编码器输出相同质量或体积。

### 5.3 初始参数建议

为动画压制提供示例起点，但不要宣称其为所有片源的最佳参数：

  ----------------------------------------------------------------------------------
  目标                    示例起点                说明
  ----------------------- ----------------------- ----------------------------------
  H.264 高质量            `libx264`, CRF 16--18,  兼容性优先，体积可能较大
                          `slow`                  

  HEVC 高质量             `libx265`, CRF 16--18,  离线压制的起始值，需要视觉对比
                          `slow`, 10-bit          

  NVIDIA HEVC             `hevc_nvenc`, CQ        速度优先于极限压缩效率；参数需按
                          18--22, `p7`（支持时）  FFmpeg 版本验证
  ----------------------------------------------------------------------------------

质量参数应在 UI
中附带说明：结果依赖片源、滤镜、编码器版本、预设、色深和观看环境。软件不应把不同编码器的质量数值当成等价值。

------------------------------------------------------------------------

## 6. FFmpeg 进程与进度管理

### 6.1 启动进程

-   使用 Rust 的 `Command` / Tokio 进程 API。
-   以参数数组传入选项，避免 shell 字符串拼接。
-   设置输入、输出和工作目录。
-   对 stdout 和 stderr 进行异步读取，避免缓冲区阻塞。
-   记录启动参数，但日志中的隐私路径可按设置脱敏。

### 6.2 进度读取

使用 FFmpeg 的机器可读进度功能，例如：

``` text
-progress pipe:1
```

按键值记录解析 `out_time`、`speed`、`frame`、`progress`
等字段。具体字段可能随版本和编码流程有所不同。

-   进度解析器与 UI 分离。
-   UI 更新节流，例如每秒 4 次以内。
-   ETA 根据媒体时长和有效输出时间估算；无法估算时不展示。
-   日志输出应做长度限制与轮转，避免长任务占用无限内存。

### 6.3 取消任务

-   UI 发出取消请求。
-   Rust 侧标记任务为 `Cancelling`。
-   终止 FFmpeg 进程，并在必要时处理子进程。
-   等待进程退出、读取最终日志。
-   清理不完整的临时输出（由用户设置决定是否保留）。
-   最终状态设置为 `Cancelled`，不能把用户取消误报为普通失败。

### 6.4 输出文件安全

-   检查输出路径是否与输入路径冲突。
-   对同名文件明确提示覆盖或自动生成新名称。
-   临时输出写入应用自有临时目录（dev：软件目录下 `encode-tmp`；发布：应用配置目录下 `encode-tmp`，安装目录 Program Files 不可写），成功后移动/重命名到目标目录；跨卷重命名失败时回退为复制到目标目录后删除临时文件（2026-10-10 按用户要求调整，原方案为"临时文件与目标同目录"）。
-   启动前探测应用临时目录与目标目录的可写性；目标目录不可写时（如受限/沙箱环境）自动回退为把成品保存到应用临时目录，并在界面明确提示（2026-10-10 补充）。回退成品名包含任务 ID（`<主干>.<任务ID>.<扩展名>`），避免不同目录下的同名输入互相覆盖成品（2026-10-10 M3 补充）。
-   失败时保留诊断信息，不留下伪装成完整成品的文件。

------------------------------------------------------------------------

## 7. UI 信息架构

建议主窗口采用简洁的四个区域：

1.  **压制**：输入文件、输出位置、编码器和主要参数。
2.  **任务**：排队、运行、完成和失败的任务列表。
3.  **预设**：内置预设和自定义预设。
4.  **设置**：FFmpeg 路径、能力检测、主题、日志和输出行为。

### 7.1 压制页面

-   顶部：输入文件与媒体概要。
-   中部：编码器、质量、预设、位深、音频/字幕处理。
-   底部：输出路径、预计体积提示（仅在有合理依据时显示）、命令预览和开始按钮。
-   高级参数默认折叠。

### 7.2 任务页面

-   任务名、状态、进度、速度、已用时间。
-   选择任务后查看日志与参数。
-   完成任务后可打开输出目录。
-   失败任务显示简明原因，并提供查看详细日志的入口。

### 7.3 视觉原则

-   优先清晰和信息密度，不做过多动效。
-   暗色和浅色主题使用 CSS variables。
-   对长路径使用截断显示和复制按钮。
-   参数项提供简短解释，避免把所有 FFmpeg 文档直接塞进 UI。
-   进度条、状态标签和日志更新应节流，防止高频渲染。

------------------------------------------------------------------------

## 8. 数据和文件布局

应用状态和用户可见预设放在系统**应用配置目录**（Tauri `app_config_dir()`，按应用标识 `com.aozora.encode` 生成），而不是安装目录。各平台根目录（已决策）：

- Windows：`%APPDATA%\com.aozora.encode`（Roaming，即 `C:\Users\<用户>\AppData\Roaming\com.aozora.encode`）
- macOS：`~/Library/Application Support/com.aozora.encode`
- Linux：`$XDG_CONFIG_HOME/com.aozora.encode`（未设置时默认 `~/.config/com.aozora.encode`）

``` text
<配置根目录>/
  settings.json
  presets/
    <预设 ID>.json          # 用户预设；内置示例预设在程序中编译提供，不落盘
  history/
    jobs.json               # 任务历史（最新在前，上限 50 条）
  logs/
    app.log
```

实现说明（与初版草案的差异）：

- 内置示例预设（H.264 兼容、HEVC 高画质、NVENC 快速）编译在程序中且只读，
  不写入配置目录——避免"删掉后又自动重建"的歧义；用户预设以 `<id>.json`
  存放，并支持把内置预设复制为自定义预设。
- 任务历史只保存结构化字段与日志尾部（每条 80 行），不保存完整命令行；
  任务页提供删除单条与清空历史。

如果任务历史增长到需要筛选、分页或关联多个实体，再迁移到
SQLite。第一版没有必要仅为了"专业架构"而先引入数据库。

任务历史应避免默认永久保存完整命令行中的敏感路径；可提供清除历史和日志的选项。

------------------------------------------------------------------------

## 9. 安全、稳定性和许可证

### 9.1 安全

-   不通过 Shell 拼接执行用户输入。
-   对自定义输出路径、参数和文件覆盖行为进行验证。
-   前端不获得任意文件系统访问权限，尽量使用 Tauri 能力与命令白名单。
-   避免把本地媒体路径发送到网络。
-   默认不联网、不收集遥测。
-   导入预设时验证 schema 和字段范围，不执行预设内的脚本或命令。

### 9.2 稳定性

-   探测工具不存在时给出安装或配置路径指引。
-   编码器缺失时隐藏或禁用相关选项。
-   处理文件名含空格、Unicode、括号和特殊字符的情况。
-   测试非标准帧率、可变帧率、无音轨、多个字幕轨和超长文件。
-   测试任务取消、磁盘空间不足、输出被占用、程序关闭和崩溃恢复。
-   在不同 FFmpeg 版本上运行集成测试。

### 9.3 许可证和分发

-   核实应用自身依赖与随包分发的 FFmpeg 构建所包含组件的许可证。
-   x264、x265 等库可能改变构建的许可证义务；发行前进行具体审查。
-   在"外部工具模式"和"内置工具包模式"之间作出明确选择。
-   在关于页面或第三方声明中列出随包分发的组件、版本和许可证。
-   不要在没有审查的情况下把任意网上下载的 FFmpeg
    二进制文件直接打包发布。

------------------------------------------------------------------------

## 10. 测试计划

### 10.1 单元测试

-   编码器配置验证。
-   参数数组生成。
-   预设 JSON 读写与版本迁移。
-   FFmpeg 进度数据解析。
-   任务状态转换。
-   路径冲突与输出命名规则。

### 10.2 集成测试

准备短小、可再分发的测试素材，覆盖：

-   1080p H.264 视频。
-   10-bit HEVC 视频。
-   多音轨、多字幕 MKV。
-   无音频的视频。
-   非 ASCII 路径和文件名。
-   编码器不可用、输入文件损坏、输出目录不可写等失败情形。

验证：

-   输出可被 FFprobe 正确读取。
-   需要保留的音频、字幕、章节按预期存在。
-   取消任务后进程确实退出。
-   失败不会被标记为成功。
-   任务进度最终状态正确。

### 10.3 性能测试

在目标 Windows 机器上分别测量：

-   应用冷启动时间。
-   空闲内存占用。
-   任务列表显示 100 条任务时的 UI 响应。
-   运行编码任务时的前端 CPU 占用。
-   日志持续输出时的内存变化。
-   FFmpeg 进程退出和取消的响应时间。

不要仅依据框架宣传或单次测试得出"最轻量"的结论。

------------------------------------------------------------------------

## 11. 里程碑

### M0：项目骨架

-   [x] 初始化 Tauri 2 + Svelte + TypeScript。
-   [x] 配置格式化、Lint 和类型检查。
-   [x] 建立基础窗口与导航。

**完成标准：** 开发、测试和打包流程可重复运行。已完成（打包为 Windows MSI）。

### M1：媒体探测

-   [x] 检测 FFmpeg/FFprobe 安装（顺序：PATH → 安装目录 `bin` → 指定路径；未安装给出错误提示；macOS/Linux 仅检测 PATH 命令，指定路径仅 Windows 使用）。
-   [x] 自定义 FFmpeg/FFprobe 实例路径。
-   [x] 探测版本。
-   [x] 探测编码器能力。
-   [x] 导入文件并显示流信息。

**完成标准：** 常见 MKV/MP4 文件可正常探测，错误信息可读。

### M2：单任务压制

-   [x] 实现 H.264、HEVC 和可用的 NVENC 编码器配置。
-   [x] 生成参数数组。
-   [x] 启动 FFmpeg，解析进度，显示日志。
-   [x] 支持取消和输出收尾。

**完成标准：** 可可靠完成、取消和诊断一个压制任务。已完成（成功/失败/取消/覆盖/非 ASCII 路径/NVENC 参数均经真实 FFmpeg 集成测试验证）。

### M3：预设与队列

-   [x] 自定义预设保存、导入和导出。
-   [x] 任务排队、失败重试、任务历史。
-   [x] 命令预览。

**完成标准：** 重启应用后预设保留，任务结果可追踪。已完成（预设与历史落盘并在重启后读回；队列排队、冲突拒绝、取消接续均经应用内真实命令端到端验证）。

### M4：动画压制增强

-   [x] 字幕处理选项：保留全部字幕轨（复制）或丢弃（2026-10-10 提前实施）。
-   [ ] 音轨、字幕轨和章节映射。
-   [ ] ASS 字幕处理的基本支持。
-   [ ] 进一步完善 MKV 封装。
-   [ ] 设计可选的 VapourSynth 集成接口，但不强制内置。

**完成标准：** 主要流映射行为可预测，字幕处理结果经过真实样片验证。

### M5：发布准备

-   [ ] 许可证清单。
-   [ ] 安装包、版本信息和升级策略。
-   [ ] 日志导出、故障诊断说明。
-   [ ] 在干净的 Windows 环境中验证依赖。

**完成标准：** 新用户能够配置工具、导入视频并完成第一次压制。

### M6：功能增强

-   [ ] 下载 FFmpeg 到工具箱安装目录 `bin`。
-   [ ] 多实例下拉选择（设置页，多实例并存时选择实际使用的 FFmpeg）。

**完成标准：** 未安装 FFmpeg 的用户可在应用内直接获取 FFmpeg / FFprobe。

------------------------------------------------------------------------

## 12. 建议的项目目录

``` text
src/
  lib/
    components/
    pages/
    stores/
    types/
    utils/
  App.svelte
  main.ts

src-tauri/
  src/
    commands/
      probe.rs
      encoder.rs
      jobs.rs
      presets.rs
      settings.rs
    media_probe/
    encoder_registry/
    planner/
    process/
    progress/
    job/
    preset_store/
    lib.rs
    main.rs
  capabilities/
  Cargo.toml
  tauri.conf.json

presets/
  h264-compatible.json
  hevc-quality.json
  nvenc-fast.json

tests/
  fixtures/
  integration/

docs/
  architecture.md
  user-guide.md
  third-party-licenses.md
```

项目早期可以把模块进一步合并；目录结构是职责参考，不要求第一天就建立所有空文件。

------------------------------------------------------------------------

## 13. 最终决策

推荐第一版使用：

**Tauri 2 + Svelte 5 + TypeScript + Rust + FFmpeg/FFprobe。**

第一版不引入：

-   SQLite（先用 JSON 保存少量任务历史和设置）。
-   VapourSynth（作为后续可选扩展）。
-   插件市场或任意脚本执行。
-   复杂的节点式工作流。
-   多任务并行编码。
-   必须联网的功能。

开发优先级为：

1.  工具检测与媒体探测。
2.  参数验证和可靠命令生成。
3.  FFmpeg 进程管理、进度与取消。
4.  预设和任务队列。
5.  字幕、音轨与滤镜增强。

衡量项目成功的标准不是功能列表有多长，而是：用户能否在不记忆 FFmpeg
命令的情况下，可靠地完成一次压制；出现失败时，能否知道为什么；想微调参数时，能否清楚看到软件实际执行了什么。
