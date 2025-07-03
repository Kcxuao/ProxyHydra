# ProxyHydra

一个用 **Rust** 编写的轻量级网络代理抓取、验证与质量评估系统。该工具支持从多个公开代理源抓取 IP，自动检测其可用性，并根据延迟、成功率、稳定性进行评分，最终存入本地 SQLite 数据库。

## ✨ 功能特点

- 支持多个代理源（如 bfbke、快代理）
- 并发验证代理可用性，限流控制
- 多轮测速，评估代理响应速度、成功率和稳定性
- 自动打分（0.0 - 1.0），用于排序或筛选优质代理
- 数据持久化存储至 `SQLite`，支持唯一索引

---

## 📦 项目依赖

- [tokio](https://docs.rs/tokio) 异步运行时
- [reqwest](https://docs.rs/reqwest) 网络请求库
- [sqlx](https://docs.rs/sqlx) 异步数据库支持
- [serde](https://serde.rs/) 与 [serde_json](https://docs.rs/serde_json) 用于序列化
- [tracing](https://docs.rs/tracing) 日志系统
- [anyhow](https://docs.rs/anyhow) 错误处理
- [thiserror](https://docs.rs/thiserror) 自定义错误枚举
- [once_cell](https://docs.rs/once_cell) 全局配置加载
- [config](https://docs.rs/config) 用于读取配置文件

---

## 🚀 快速开始

### 1. 克隆项目

```bash
git clone https://github.com/yourname/rust-proxy-crawler.git
cd rust-proxy-crawler
```

### 2. 添加配置文件

在项目根目录创建 `Config.toml` 文件：

```toml
semaphore = 20
timeout = 5
```

> `semaphore`：最大并发任务数
> `timeout`：单个代理请求超时时间（单位：秒）

### 3. 构建 & 运行

```bash
cargo build --release
cargo run --release
```

启动后程序将会：

1. 抓取公开代理源中的 IP；
2. 对每个代理进行 3 次测速；
3. 根据速度、成功率和稳定性计算综合评分；
4. 保存至 `proxy.db` 数据库中。

---

## 📁 项目结构

```
.
├── main.rs              // 主程序入口
├── model.rs             // 数据模型定义 (Proxy, Config)
├── fetcher.rs           // 抓取器模块
├── verifier.rs          // 验证器模块（含评分逻辑）
├── quality.rs           // 质量评估模块
├── storage.rs           // 数据库存储逻辑
├── error.rs             // 错误类型定义
├── Config.toml          // 配置文件（需手动创建）
└── proxy.db             // SQLite 数据库（首次运行后自动创建）
```

---

## 📊 代理评分规则

```text
综合评分 (score) = 
    0.4 * 响应速度评分 +
    0.3 * 请求成功率 +
    0.3 * 稳定性
```

响应速度评分规则：

| 平均响应时间(ms) | 得分  |
| ---------- | --- |
| < 100      | 1.0 |
| < 500      | 0.8 |
| < 1000     | 0.5 |
| < 2000     | 0.3 |
| >= 2000    | 0.1 |

---

## 🛠 数据库结构

```sql
CREATE TABLE proxy (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ip TEXT NOT NULL,
  port TEXT NOT NULL,
  speed REAL,
  success_rate REAL,
  stability REAL,
  score REAL,
  last_checked TEXT,
  UNIQUE(ip, port)
);
```

---

## 🧪 单元测试

```bash
cargo test
```

测试覆盖配置加载与数据库插入验证等基础功能。

---

## 📌 TODO / 未来改进

* [ ] 增加更多代理源（如 89ip、国内国外混合源等）
* [ ] Web UI 管理界面（Rocket + Vue/React）
* [ ] 支持 SOCKS5/HTTPS 代理类型判断
* [ ] 数据库迁移工具或分页接口

---

## 📄 License

MIT License © 2025 Kcxuao

