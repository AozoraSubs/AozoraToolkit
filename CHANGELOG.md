## v0.2.1

### 新增 | New

* 压制页文件导入与媒体信息展示，设置页能力检测展示 @ImJingLan
* FFmpeg 能力检测与 FFprobe 媒体探测 @ImJingLan
* dev 构建使用可执行文件旁的数据目录以适配工作区沙箱 @ImJingLan
* 设置页展示 FFmpeg 检测结果与自定义路径 @ImJingLan
* 添加 FFmpeg/FFprobe 检测与设置模型核心 crate @ImJingLan
* 初始化 Tauri 2 + Svelte 5 项目骨架 @ImJingLan

### 修复 | Fix

* 恢复工具脚本被错误编码破坏的中文内容 @ImJingLan
* 修复周快照工作流中文注释被错误编码破坏 @ImJingLan
* 恢复被脚手架模板覆盖的 README 并补充开发环境说明 @ImJingLan

### 改进 | Improved

* DTO 统一 camelCase 并补充能力检测实机测试 @ImJingLan

### 文档 | Docs

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

* 大版本发布改为先升版本后构建再打标签 @ImJingLan
* 大版本与周快照发布并入三平台构建 @ImJingLan
* 添加三平台构建验证工作流 @ImJingLan
* 添加 GitHub Actions 基础检查工作流 @ImJingLan

### 其他 | Other

* 添加发布与快照 changelog 工具 @ImJingLan
* 移除 CI 工作流，CI 移出项目范围 @ImJingLan
