## 介绍
> 一个基于Rust和Amis.js/React的后台管理系统起始模版

## 常用命令

### 后端
- 启动服务: `cargo run --bin server`
- 代码检查: `cargo clippy --workspace`
- 运行测试: `cargo test --workspace`

### 前端
- 安装依赖: `pnpm install`
- 启动开发服务器: `pnpm dev`
- 构建生产版本: `pnpm build`

## 系统架构

```
oxide_admin/
├── app/                    # Rust后端
│   ├── adapter/           # API层 (REST端点)
│   ├── application/       # 应用层 (用例/服务)
│   ├── domain/           # 领域层 (实体/值对象)
│   ├── infrastructure/   # 基础设施层 (仓库实现)
│         └── implementation/   # 领域实现
│         └── migration/        # 数据库迁移
│         └── repository/       # 仓储实现
├── frontend/             # 前端应用
├── target/              # 构建输出
└── Cargo.toml           # workspace配置
```

### 技术栈
- **后端**: Rust + Axum + SQLx + Nject + SQLite + Redb
- **前端**: TypeScript + Rsbuild
- **架构**: 领域驱动设计(DDD) + 整洁架构
