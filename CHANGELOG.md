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
