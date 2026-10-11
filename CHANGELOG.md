## 0.3.0

### 新增 | New

* 产品名统一为 Aozora Toolbox @ImJingLan
* 侧边栏品牌名改为 Aozora Toolbox 并标注版权 @ImJingLan
* 侧边栏品牌标识改用项目图标 @ImJingLan
* 替换应用图标为项目图标 @ImJingLan
* 预设页、任务页与压制页对接队列与命令复制 @ImJingLan
* 新增前端预设模型、共享表单状态与命令展示工具 @ImJingLan
* 新增预设、任务队列与任务历史命令层 @ImJingLan
* 新增单任务队列状态机 @ImJingLan
* 新增任务历史持久化存储 @ImJingLan
* 新增预设模型与本地预设存储 @ImJingLan
* 压制页添加外部字幕烧录选项 @ImJingLan
* 支持烧录外部字幕文件到画面 @ImJingLan
* 任务状态全局化并在会话内缓存探测结果 @ImJingLan
* 隐藏 FFmpeg 子进程控制台窗口并异步化探测命令 @ImJingLan
* 压制页添加字幕处理选项 @ImJingLan
* 支持保留或丢弃字幕轨的压制选项 @ImJingLan
* 目标目录不可写时输出应用进程可写性自检报告 @ImJingLan
* 目标目录不可写时自动回退保存到应用临时目录 @ImJingLan
* 临时输出写入应用自有临时目录并在成功后移动到目标 @ImJingLan
* 压制页支持编码设置、任务进度与日志 @ImJingLan
* 新增压制任务命令与事件 @ImJingLan
* 实现 FFmpeg 进度解析与编码运行器 @ImJingLan
* 定义编码配置模型并生成 FFmpeg 参数 @ImJingLan

### 修复 | Fix

* 预设下拉框保留所选预设并标注参数修改 @ImJingLan
* 修正前端任务状态与历史读取的边界处理 @ImJingLan
* 加固队列数据交接、回退路径唯一性与请求校验 @ImJingLan
* 自动清理只读残留临时输出并允许覆盖只读旧输出 @ImJingLan
* 输出不可写时在任务启动前快速失败并给出可读提示 @ImJingLan
* 为 Windows 产物内嵌 Common-Controls v6 清单 @ImJingLan
* 发布仅上传最终安装包，排除打包中间文件 @ImJingLan
* 修复发布工作流 needs 缺失导致的空 tag 错误 @ImJingLan

### 文档 | Docs

* 修正 README 顶部结构 @ImJingLan
* 记录产品名统一与版本号升级 @ImJingLan
* README 改用带图标的居中标题 @ImJingLan
* 记录侧边栏品牌标识与前端图标资源 @ImJingLan
* 记录应用图标替换与重新生成方式 @ImJingLan
* 记录 M3 预设与队列完成情况 @ImJingLan
* 记录外部字幕烧录功能完成 @ImJingLan
* 记录 MSVC 静态链接达成与清华源配置 @ImJingLan
* 记录体验修复批与 MSVC 切换的当前状态 @ImJingLan
* 记录字幕选项提前实施并更新清单 @ImJingLan
* 记录解除工作区低完整性标记的后续处理 @ImJingLan
* 记录受限工作区的低完整性标记问题与修复命令 @ImJingLan
* 记录应用进程低完整性的根因与修复命令 @ImJingLan
* 更新 PLAN 与 README 完成 M2 @ImJingLan

### 其他 | Other

* 添加仓库根目录项目图标资源 @ImJingLan
* 覆盖 NVENC 编码器参数的真实 FFmpeg 集成验证 @ImJingLan

## 0.3.4

### 新增 | New

* 常用分辨率补充 4K 档位 @ImJingLan
* 新增输出分辨率调整 @ImJingLan
* 补齐 AV1 软件编码器（SVT-AV1 / libaom / rav1e） @ImJingLan
* 按实际环境检测可用编码器与显卡 @ImJingLan
* 新增封装格式选择器（MKV / MP4 / WebM） @ImJingLan
* 产品名统一为 Aozora Toolbox @ImJingLan
* 侧边栏品牌名改为 Aozora Toolbox 并标注版权 @ImJingLan
* 侧边栏品牌标识改用项目图标 @ImJingLan
* 替换应用图标为项目图标 @ImJingLan
* 预设页、任务页与压制页对接队列与命令复制 @ImJingLan
* 新增前端预设模型、共享表单状态与命令展示工具 @ImJingLan
* 新增预设、任务队列与任务历史命令层 @ImJingLan
* 新增单任务队列状态机 @ImJingLan
* 新增任务历史持久化存储 @ImJingLan
* 新增预设模型与本地预设存储 @ImJingLan
* 压制页添加外部字幕烧录选项 @ImJingLan
* 支持烧录外部字幕文件到画面 @ImJingLan
* 任务状态全局化并在会话内缓存探测结果 @ImJingLan
* 隐藏 FFmpeg 子进程控制台窗口并异步化探测命令 @ImJingLan
* 压制页添加字幕处理选项 @ImJingLan
* 支持保留或丢弃字幕轨的压制选项 @ImJingLan
* 目标目录不可写时输出应用进程可写性自检报告 @ImJingLan
* 目标目录不可写时自动回退保存到应用临时目录 @ImJingLan
* 临时输出写入应用自有临时目录并在成功后移动到目标 @ImJingLan
* 压制页支持编码设置、任务进度与日志 @ImJingLan
* 新增压制任务命令与事件 @ImJingLan
* 实现 FFmpeg 进度解析与编码运行器 @ImJingLan
* 定义编码配置模型并生成 FFmpeg 参数 @ImJingLan
* 应用信息支持快照版本编译期注入 @ImJingLan
* 压制页文件导入与媒体信息展示，设置页能力检测展示 @ImJingLan
* FFmpeg 能力检测与 FFprobe 媒体探测 @ImJingLan
* dev 构建使用可执行文件旁的数据目录以适配工作区沙箱 @ImJingLan
* 设置页展示 FFmpeg 检测结果与自定义路径 @ImJingLan
* 添加 FFmpeg/FFprobe 检测与设置模型核心 crate @ImJingLan
* 初始化 Tauri 2 + Svelte 5 项目骨架 @ImJingLan

### 修复 | Fix

* 统一以 LF 检出，修掉仅 windows-2022 失败的 format:check @ImJingLan
* Changelog 基准固定为上一个正式 Release 且不再覆盖历史 @ImJingLan
* 修复发布流程因版本提交缺少检查记录而卡死 @ImJingLan
* version string @ImJingLan
* 预设下拉框保留所选预设并标注参数修改 @ImJingLan
* 修正前端任务状态与历史读取的边界处理 @ImJingLan
* 加固队列数据交接、回退路径唯一性与请求校验 @ImJingLan
* 自动清理只读残留临时输出并允许覆盖只读旧输出 @ImJingLan
* 输出不可写时在任务启动前快速失败并给出可读提示 @ImJingLan
* 为 Windows 产物内嵌 Common-Controls v6 清单 @ImJingLan
* 发布仅上传最终安装包，排除打包中间文件 @ImJingLan
* 修复发布工作流 needs 缺失导致的空 tag 错误 @ImJingLan
* macOS 产物签名验证改为挂载 dmg 校验并清理死代码警告 @ImJingLan
* 恢复工具脚本被错误编码破坏的中文内容 @ImJingLan
* 修复周快照工作流中文注释被错误编码破坏 @ImJingLan
* 恢复被脚手架模板覆盖的 README 并补充开发环境说明 @ImJingLan

### 改进 | Improved

* 检查命令抽成复合动作并去掉三层工作流嵌套 @ImJingLan
* DTO 统一 camelCase 并补充能力检测实机测试 @ImJingLan

### 文档 | Docs

* 修正 README 顶部结构 @ImJingLan
* 记录产品名统一与版本号升级 @ImJingLan
* README 改用带图标的居中标题 @ImJingLan
* 记录侧边栏品牌标识与前端图标资源 @ImJingLan
* 记录应用图标替换与重新生成方式 @ImJingLan
* 记录 M3 预设与队列完成情况 @ImJingLan
* 记录外部字幕烧录功能完成 @ImJingLan
* 记录 MSVC 静态链接达成与清华源配置 @ImJingLan
* 记录体验修复批与 MSVC 切换的当前状态 @ImJingLan
* 记录字幕选项提前实施并更新清单 @ImJingLan
* 记录解除工作区低完整性标记的后续处理 @ImJingLan
* 记录受限工作区的低完整性标记问题与修复命令 @ImJingLan
* 记录应用进程低完整性的根因与修复命令 @ImJingLan
* 更新 PLAN 与 README 完成 M2 @ImJingLan
* 同步 AGENTS 的 CI 检查与构建说明 @ImJingLan
* 更新 AGENTS 的 CI 检查与构建说明 @ImJingLan
* 版本号仅在用户要求时更新，由代理综合决定更新幅度 @ImJingLan
* 定义版本号更新机制并写入贡献规范 @ImJingLan
* 记录 M1 完成情况并更新里程碑状态 @ImJingLan
* 记录 dev 数据目录方案解决工作区沙箱限制 @ImJingLan
* 更新 M1 完成记录说明 GUI 验证方式与环境限制 @ImJingLan
* 更新 PLAN 记录 M1 安装检测完成情况 @ImJingLan
* 记录各平台配置目录决策（Windows 使用 %APPDATA%） @ImJingLan
* PLAN 增加自定义 FFmpeg 路径与多实例下拉选择计划 @ImJingLan
* README 功能清单按里程碑分组，应用内下载 FFmpeg 单列于 M6 @ImJingLan
* 更新 PLAN 增加 M1 安装检测与 M6 下载 FFmpeg 计划 @ImJingLan
* 说明 macOS 未公证产物的打开方式 @ImJingLan
* 要求完成变更后主动提交 @ImJingLan
* 更新平台范围说明以匹配 CI 构建验证 @ImJingLan
* 更新 PLAN 记录 M0 完成情况 @ImJingLan
* 明确项目仅面向 Windows 桌面端的范围 @ImJingLan
* 添加项目规划与贡献规范文档 @ImJingLan

### 自动化 | CI

* 拆分独立检查工作流，构建前校验提交检查结果 @ImJingLan
* 工作流步骤名称统一为中文 @ImJingLan
* 周快照构建注入快照版本号到二进制与产物文件名 @ImJingLan
* push 触发改为只检查不打包，发布流程执行完整打包 @ImJingLan
* 大版本发布改为先升版本后构建再打标签 @ImJingLan
* 大版本与周快照发布并入三平台构建 @ImJingLan
* 添加三平台构建验证工作流 @ImJingLan
* 添加 GitHub Actions 基础检查工作流 @ImJingLan

### 其他 | Other

* 修正用例名拼写 @ImJingLan
* 添加仓库根目录项目图标资源 @ImJingLan
* 覆盖 NVENC 编码器参数的真实 FFmpeg 集成验证 @ImJingLan
* Merge branch 'main' of https://github.com/AozoraSubs/AozoraToolkit @ImJingLan
* 添加发布与快照 changelog 工具 @ImJingLan
* 移除 CI 工作流，CI 移出项目范围 @ImJingLan
