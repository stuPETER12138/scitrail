# 科迹 SciTrail

**面向博士生的本地优先科研轨迹与任务管理器** · A local-first research trajectory & task manager for PhD students.

在同一个本机应用里管理课题、任务、DDL、实验记录与周报：轨迹页把一切"已发生的事"投影成按日分组的账本，回顾页用热力图和连续记录天数让"天天记录"看得见。数据全部存在本地 SQLite 单文件里，随时带走，不上传任何地方。

## 功能路线

- [x] P0 骨架：口令解锁（argon2）· 课题读取 · 托盘常驻 · 全局速录小窗（Ctrl+Alt+K）
- [x] P1 任务：任务 / 收件箱 / 看板（拖拽）· 速录语法（`实 完成#催化 1150℃×2h 明天 9:00`）· 课题管理 · 重复任务 · 周统计
- [ ] P2 时间：DDL 与学位里程碑 · 倒计时（提前 7/3/1 天）
- [ ] P3 轨迹：实验 / 文献 / 报销 / 组会 / 笔记条目 · 账本 + 当日时序条 + 检查器
- [ ] P4 周报：一键生成本周汇报草稿
- [ ] P5 安全网：每日自动备份（`VACUUM INTO` 一致性快照）· 全量导出 JSON/Markdown

设计文档见 [docs/](docs/)：需求报告、五方案设计对比、开发设计说明书（数据库 DDL / 命令清单 / 里程碑拆解）。

## 技术栈

Tauri 2（Rust 内核 + WebView2）· rusqlite(bundled) + SQLite WAL + FTS5 · Vue 3 + Vite + TypeScript + Pinia · 不引重型 UI 库

## 开发

```bash
pnpm install
pnpm tauri dev      # 开发（前端热重载 + Rust 增量编译）
pnpm tauri build    # 打包 Windows 安装包（NSIS）
```

Rust 测试（含 FTS5 可用性验证）：

```bash
cd src-tauri && cargo test
```

环境要求：Node 24+（开发机为 26 亦可）· pnpm · Rust stable（MSVC 工具链）。

## License

[MIT](LICENSE)
