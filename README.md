> 一个基于Rust和Amis.js/React的后台管理系统起始模版

[![Build](https://github.com/shmy/oxide_admin/actions/workflows/build.yaml/badge.svg)](https://github.com/shmy/oxide_admin/actions/workflows/build.yaml)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)

## 🎯 项目目标
- 提供一个快速构建后台管理系统的起点
- 使用现代化的Rust和Amis.js/React技术栈
- 遵循领域驱动设计(DDD)和整洁架构原则

## 👀 在线预览
> 由于使用 `Render`的免费计划，访问可能较慢，15分钟无操作会冻结实例，之后访问需要经过`Render`的中间页，请知悉。

[https://oxide-admin.onrender.com/_](https://oxide-admin.onrender.com/_)
> 请勿修改密码

- 账号：admin
- 密码：123456


## ✨ 特性
- DDD架构：遵循领域驱动设计(DDD)和整洁架构原则，实现适配展示、应用服务、领域模型、基础设施层的分离；
- 事件总线：内置事件系统，通过发布/监听领域事件来解耦业务逻辑；
- 依赖注入：由`nject` crate进行支持；
- 代码生成：一键生成各个模块的代码，诸如`CRUD`、`CommandHandler`、`QueryHandler`等等；
- 时区配置：配置数据库时区；
- 用户认证：使用`JWT`，支持`refresh_token`和`access_token`的签发、验证和刷新；
- 用户授权：内置`RBAC`，灵活的控制前端菜单权限以及接口权限验证；
- 数据库自动迁移：部署时无需手动迁移；
- 速率限制中间件: 可对路由进行限速；
- 图形验证: 防止暴力破解，防止恶意请求；
- 日志轮转、日志分级：由`tracing` crate进行支持；
- 内建*single_flight*宏：缓解数据库压力；
- 文件上传及访问签名：内建单文件上传、图片上传、分片上传等接口，适配`Amis`；
- KV缓存：支持`ttl`，使用`redis`或`redb`；
- 后台任务：由`faktory` 使用支持；
- 优雅关停：严谨地结束服务、释放资源；
- 多源配置：支持环境变量、`.env`文件，或者使用`cli`参数；
- Github CI：自动构建`x86_64-unknown-linux-musl`；
- ...

### Crate features
- `kv_redb`: 使用`redb`作为kv/缓存，适合单体项目，默认值；
- `kv_redis`: 使用`redis`作为kv/缓存，适合分布式项目；
- `bg_dummy`: 禁止使用后台任务，默认值；
- `bg_faktory`: 使用`faktory`作为后台任务；
- `object_storage_fs`: 使用本地文件系统作为对象存储，默认值；
- `object_storage_s3`: 使用S3兼容服务作为对象存储；

## 🎈前端
- 架构：使用`Amis`低代码，借用其丰富的组件，快速地完成的CRUD相关功能，也可以通过React自定义组件进行补充；
- 优化：构建时自动混淆、自动gzip压缩；

## ⚙️ 技术栈
- **后端**: Rust + Axum + SQLx + Nject + Postgres + Redb
- **前端**: Amis.js + React + TypeScript + Rsbuild
- **工具**: just + pnpm


## 📁 目录结构

```txt
oxide_admin/
├── app/                    # Rust后端
│   ├── adapter/            # API层 (REST端点)
│   ├── application/        # 应用层 (用例/服务)
│   ├── domain/             # 领域层 (实体/值对象)
│   ├── infrastructure/     # 基础设施层 (技术实现)
│         └── port/             # 领域实现
│         └── migration/        # 数据库迁移
│         └── repository/       # 仓储实现
├── frontend/             # 前端应用
├── target/               # 构建输出
└── Cargo.toml            # workspace 配置
```
> 严格遵守`DDD`设计原则，确保代码的可维护性和可扩展性。

## 🛠️ 快速开始
> 请先确保已安装 Rust 和 Node.js 环境，以及 just 和 pnpm。

### 克隆项目及初始化
```bash
git clone git@github.com:shmy/oxide_admin.git
cd oxide_admin
# start a postgres
docker compose up -d
# setup env
cp .env.example .env
# install sqlx-cli & cargo-watch
cargo install sqlx-cli cargo-watch
# setup sqlx migration
just setup
```
### 启动后端
```base
just dev
```
> 后端默认监听`127.0.0.1:8080`，前端会有`dev server`进行代理；

### 启动前端
```base
cd frontend
pnpm install
pnpm dev
```
> 访问 `http://127.0.0.1:3000/_`

## 📦 构建命令
- 本机架构:
```bash
just build
```
- 交叉编译：`Linux/x86_64-unknown-linux-musl`
> 需要确保安装了`cross`, 使用`cargo install cross`命令进行安装。
```bash
just build_linux_x86_64_musl
```
- 交叉编译：`Windows/x86_64-pc-windows-msvc`
> 需要确保安装了`xwin`, 使用`cargo install cargo-xwin`命令进行安装。
```bash
just build_windows_x86_64_msvc
```
- 编译Docker image
```bash
just build_container
```

## 📃 代码生成
### CRUD 生成
```bash
cargo g scaffold -h
```

### 更多详见
```bash
cargo g -h
```
