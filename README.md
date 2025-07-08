# 🐍 ProxyHydra

> **一站式高性能代理池管理工具**  
> 抓取、验证、评估、存储 —— 你的网络代理质量管家 🚀

---

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![License](https://img.shields.io/github/license/yourusername/proxyhydra)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![Database](https://img.shields.io/badge/DB-SQLite%20%7C%20MySQL%20%7C%20PostgreSQL-blue)
![Async](https://img.shields.io/badge/async_tokio-✔️-informational)

---



## 📌 项目简介

**`ProxyHydra`** 是一个用 Rust 编写的高性能、模块化的网络代理抓取和质量管理框架。它致力于提供稳定、高匿名、可评估的代理服务，支持多源抓取、验证、多维打分、数据库存储及未来扩展 API 接口。

适合构建：  
✅ 高质量代理池服务  
✅ 网络爬虫代理管理组件  
✅ 自托管代理质量监测系统

---



## ✨ 功能特性

- 🌍 **多源抓取**：支持多种免费代理网站抓取
- ⚡ **异步验证**：基于 `tokio` 实现高并发非阻塞验证
- 📈 **质量评估**：根据延迟、匿名性、成功率打分评级
- 🛢️ **多数据库支持**：支持 SQLite / MySQL / PostgreSQL
- 📦 **模块解耦**：职责清晰，易于测试和扩展
- 🛠️ **统一接口**：基于 Trait 抽象存储接口，轻松适配不同数据库后端

## 



## 🧱 项目结构

```txt
src/
├─ main.rs                   # 启动入口
│
├─ common/                  # 通用模块
│   ├─ mod.rs
│   ├─ error.rs             # 错误处理封装
│   ├─ log.rs               # 日志初始化
│   └─ utils.rs             # 公共工具函数
│
├─ db/                      # 数据库相关实现
│   ├─ mod.rs
│   ├─ global.rs            # 数据库全局实例
│   ├─ manager.rs           # 数据访问管理器（Trait接口）
│   ├─ mysql.rs             # MySQL 存储实现
│   ├─ postgres.rs          # PostgreSQL 存储实现
│   └─ sqlite.rs            # SQLite 存储实现
│
├─ fetcher/                 # 代理抓取模块
│   ├─ mod.rs
│   ├─ all.rs               # 聚合抓取器
│   ├─ bfbke.rs             # 某站抓取逻辑
│   └─ kuai.rs              # 某站抓取逻辑
│
├─ model/                   # 数据模型定义
│   ├─ mod.rs
│   ├─ proxy.rs             # ProxyBasic、CheckResult 等结构体
│   └─ app_config.rs        # 应用配置结构体
│
└─ service/                 # 核心服务逻辑
    ├─ mod.rs
    ├─ verifier.rs          # 代理验证服务（异步）
    └─ quality.rs           # 代理质量评估逻辑
├── config.toml             # 配置文件
├── Cargo.toml
└── README.md
```



## 🚀 快速开始

### 1. 克隆代码

```bash
git clone https://github.com/Kcxuao/ProxyHydra.git
cd ProxyHydra
```

### 2. 安装 Rust 工具链

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 3. 构建项目

```bash
cargo build --release
```

### 4. 配置数据库连接

在 `.env` 文件或启动参数中设置 `DATABASE_URL`：

```bash
# SQLite 示例
connection_string=sqlite://proxyhydra.db

# MySQL 示例
# connection_string=mysql://user:pass@localhost:3306/proxyhydra

# PostgreSQL 示例
# connection_string=postgres://user:pass@localhost/proxyhydra
```

### 5. 启动应用

```bash
cargo run --release
```



## 🔍 模块说明

| 模块名     | 描述                                            |
| ---------- | ----------------------------------------------- |
| `fetcher`  | 从互联网上多个源异步抓取代理                    |
| `verifier` | 使用 `reqwest`+`tokio` 验证代理可达性与响应时间 |
| `quality`  | 基于多指标打分（延迟、匿名性、失败率）          |
| `storage`  | 提供 trait 接口与数据库后端实现                 |
| `model`    | 定义数据结构，如 `ProxyBasic` 与 `CheckResult`  |
| `utils`    | 公共函数，如 IP 提取、评分等级等                |
| `error`    | 封装统一错误处理逻辑                            |



## 📋 数据模型简要

```rust
struct Proxy {
    /// 代理的 IP 地址（IPv4 或 IPv6）。
    pub ip: String,

    /// 代理的端口号（字符串形式，便于处理）。
    pub port: String,

    /// 平均响应速度（单位：秒），从多个测试请求中得出。
    ///
    /// 若未进行测速，该字段为 `None`。
    pub speed: Option<f64>,

    /// 成功率，表示请求成功次数占总请求次数的比例（范围 0.0 - 1.0）。
    ///
    /// 若未进行测试，该字段为 `None`。
    pub success_rate: Option<f64>,

    /// 稳定性分数，反映响应时间的一致性（如标准差或方差反比）。
    ///
    /// 分值越高表示响应越稳定。若未测试，该字段为 `None`。
    pub stability: Option<f64>,

    /// 综合评分，基于成功率、速度和稳定性计算得出。
    ///
    /// 用于排序和筛选高质量代理。若尚未评分，则为 `None`。
    pub score: Option<f64>,

    /// 最近一次进行质量检测的时间。
    ///
    /// 若代理尚未被验证或存储前未检测，则为 `None`。
    pub last_checked: Option<NaiveDateTime>,
}
```



## 🔭 未来规划

-  ✅ REST API 接口支持
-  📊 Web 仪表盘监控页面（Salvo+ Tonic + Yew）
-  📅 定时任务调度系统（基于 cron 式语法）
-  🔍 引入机器学习优化评分模型



## 🤝 贡献方式

欢迎贡献代码、优化文档、提交 issue 或 feature request！

```bash
# Fork 并克隆
git clone https://github.com/Kcxuao/ProxyHydra.git
# 创建分支
git checkout -b feature/xxx
# 提交 PR
```



## 📄 License

MIT License



> 开发者：[@Kcxuao](https://github.com/Kcxuao/)
>  如果这个项目帮到你，请点个 ⭐️！
