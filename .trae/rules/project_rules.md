# SwallowMail 项目规则

## 技术栈

- 后端：Rust (Tauri 2.0) + imap + native-tls + lettre + rusqlite + mail-parser
- 加密：aes-gcm (AES-256-GCM) + sha2 (密钥派生) + base64 (编解码)
- 异步/工具：tokio + regex + uuid + chrono + thiserror + anyhow
- 前端：React 18 + TypeScript + Zustand + react-i18next + react-icons
- Tauri 插件：dialog, notification, opener, process, shell
- 构建：Vite + Cargo
- 存储：SQLite (rusqlite bundled)，数据库文件 `swallowmail.db`

## 项目结构

- `src-tauri/src/lib.rs` — Tauri 入口，AppState 定义，命令注册
- `src-tauri/src/commands.rs` — 所有 Tauri 命令实现，IMAP 连接池管理，规则自动应用
- `src-tauri/src/mail/model.rs` — 数据模型（MailAccount, MailFolder, MailMessage, MailRule, FolderMapping 等）
- `src-tauri/src/mail/account/mod.rs` — 账户工厂（QQ/163/126/Gmail/Outlook/自定义预设）
- `src-tauri/src/mail/folder/mod.rs` — 文件夹构建与特殊用途检测
- `src-tauri/src/mail/folder/localize.rs` — 文件夹名称本地化
- `src-tauri/src/mail/store/mod.rs` — SQLite 持久层，密码 AES-256-GCM 加密/解密
- `src-tauri/src/mail/imap/client.rs` — IMAP 连接与文件夹列表获取
- `src-tauri/src/mail/imap/sync.rs` — 邮件同步，HTML 正文 XSS 清洗
- `src-tauri/src/mail/smtp/mod.rs` — SMTP 发送
- `src/stores/mailStore.ts` — 主状态管理（Zustand），含规则/文件夹映射/统一收件箱
- `src/stores/settingsStore.ts` — 设置状态（主题/签名/列表密度/同步间隔等，persist 持久化）
- `src/components/account/` — 账户管理（添加/编辑对话框）
- `src/components/common/` — 通用组件（设置对话框/侧边栏/Toast）
- `src/components/compose/` — 撰写邮件窗口
- `src/components/mail/` — 邮件列表与阅读器
- `src/i18n/` — 国际化（zh-CN / en）
- `src/styles/global.css` — 全局样式，CSS 变量 + 暗色模式

## 已实现功能

- 多账户管理（QQ/163/126/Gmail/Outlook/自定义 IMAP）
- 账户编辑（修改显示名/密码/服务器配置）
- IMAP 文件夹同步与本地化显示
- 邮件列表（紧凑/舒适密度模式）
- 邮件阅读（HTML/纯文本切换，iframe sandbox 渲染）
- 邮件撰写与发送（附件上传，账户签名）
- 邮件操作（已读/星标/删除/移动）
- 邮件搜索与线程视图
- 统一收件箱（跨账户聚合）
- 邮件规则/过滤器（按 Subject/From/To 匹配，支持 MoveTo/MarkRead/MarkFlagged/Delete）
- 规则自动应用（同步新邮件时自动执行匹配规则）
- 自定义文件夹映射（映射 Sent/Drafts/Trash/Junk 到服务器文件夹）
- 暗色模式（CSS 变量 + prefers-color-scheme + 手动切换）
- 系统通知（新邮件桌面通知）
- 定时同步（可配置间隔）
- 密码 AES-256-GCM 加密存储（兼容旧版 enc: 前缀格式）
- IMAP 连接池（上限 10，LRU 淘汰）
- 多账户并行同步（Promise.all）
- HTML 正文 XSS 清洗（移除 script 标签和 on* 事件属性）

## 编码规范

### Rust 后端
- 新增 Tauri 命令必须在 `lib.rs` 的 `invoke_handler` 中注册
- AppState 锁定顺序：先 store 后 imap_sessions，避免死锁
- IMAP session 使用后必须通过 `return_imap_session` 归还连接池
- 数据库 Schema 变更必须向后兼容（ALTER TABLE ADD COLUMN 带 DEFAULT）
- 错误统一返回 `Result<T, String>`，使用 `map_err(|e| format!("描述: {}", e))`
- 密码存储使用 `encrypt_password`，读取使用 `decrypt_password`（自动兼容明文/旧版 enc:/新版 aes: 格式）

### TypeScript 前端
- 新增 store 方法需在 interface 和实现中同步声明
- 所有 UI 文本必须添加 i18n key（zh-CN.json + en.json）
- 组件使用函数式组件 + hooks
- 状态管理使用 Zustand，持久化设置使用 persist 中间件

## 构建验证

- Rust: `cargo check` (src-tauri/)
- TypeScript: `npx tsc --noEmit`
- 完整构建: `npm run build`

## 关键约束

- 密码以 AES-256-GCM 加密存储，密钥由 SHA-256(machine_id) 派生，兼容旧版格式自动迁移
- IMAP 连接池上限 10（IMAP_POOL_MAX），超出时 LRU 淘汰最旧连接
- HTML 正文经 sanitize_html 清洗后通过 iframe sandbox 渲染，移除 script 标签和 on* 事件属性
- 多账户同步使用 Promise.all 并行执行
- 邮件规则在同步新邮件时自动应用（apply_rules_for_new_messages）
- 文件夹映射允许用户自定义 Sent/Drafts/Trash/Junk 对应的服务器文件夹名