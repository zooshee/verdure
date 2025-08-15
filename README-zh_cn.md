# Verdure

[![Crates.io version](https://img.shields.io/crates/v/verdure.svg?style=flat-square)](https://crates.io/crates/verdure)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/verdure)

[English](./README.md) | 简体中文

Verdure - Rust 的生态框架

正如它的名字一样，Verdure 代表着一个充满活力和蓬勃发展的生态框架，致力于通过全面、集成的工具和模式套件，促进便捷高效的 Rust 开发。


目前项目处于基础开发阶段，我们期待热情的贡献者加入我们一起构建这个框架。

## 生态模块

### ✅ 当前版本 (v0.0.1) - 基础设施

- **verdure-core**: 基础类型、错误处理和通用工具
- **verdure-ioc**: 依赖注入容器和组件管理  
- **verdure-macros**: 编译时代码生成和注解处理
- **verdure-context**: 应用上下文和环境管理 (🚧)

### 🚧 即将发布 - 完整生态系统

**应用框架**：
- verdure-boot: 自动配置和应用程序引导
- verdure-config: 配置管理和属性绑定
- verdure-profiles: 环境特定的配置文件

**Web 和网络**：
- verdure-web: 具有 MVC 模式的 Web 框架
- verdure-http: HTTP 客户端和服务器抽象
- verdure-websocket: WebSocket 支持和实时通信

**数据和持久化**：
- verdure-data: 数据访问模式和仓库抽象
- verdure-orm: 具有活动记录模式的对象关系映射
- verdure-transaction: 事务管理和 ACID 支持

**安全和认证**：
- verdure-security: 认证和授权框架
- verdure-oauth: OAuth2 和 OpenID Connect 集成


## 当前特性 (v0.0.1)

- [x] **IoC 容器**: 具有自动解析的全面依赖注入
- [x] **组件生命周期**: 单例和原型作用域，带生命周期事件
- [x] **注解驱动开发**: `#[derive(Component)]` 和 `#[autowired]` 用于声明式配置
- [x] **事件系统**: 容器和组件生命周期事件处理
- [x] **循环依赖检测**: 防止无限依赖循环
- [x] **线程安全**: 多线程应用程序的完全并发访问支持

### 📋 路线图 - 构建完整生态系统

- [ ] **自动配置**: 开箱即用的应用程序引导
- [ ] **Web 框架**: MVC 模式和 REST API 开发
- [ ] **数据访问**: 仓库模式和 ORM 集成
- [ ] **安全框架**: 认证和授权
- [ ] **AOP（面向切面编程）**: 切面编程支持
- [ ] **消息驱动架构**: 事件驱动编程模式
- [ ] **可观测性**: 指标、追踪和健康检查
- [ ] 以及更多...

## 引入依赖

```toml
verdure = "0.0.1"
inventory = "0.3"
```
底层目前强依赖于 `inventory`，感谢这个优秀的 Repo。

## Context - 推荐
### 快速使用
#### Configuration 自动读取配置文件
`application.yml`示例文件:
```yaml
server:
  name: TestApp
  port: 8080
datasource:
  host: 127.0.0.1
  username: root
  password: 123456
  database: test
```
带有`Configuration`的`derive`结构体会自动注册成`Component`自动读取配置并装载，若配置文件中不存在该键值则会使用`config_default`或`config_default_t`，如果不存在默认值则为`None`,
需要注意的是字段类型必须使用`Option<T>`包装。
```rust
use std::sync::Arc;
use verdure::event::{ContextAwareEventListener, ContextInitializingEvent};
use verdure::{ApplicationContext, ComponentFactory, Configuration};

#[derive(Debug, Configuration)]
#[configuration("server")]
struct ServerConfig {
    // server.name
    name: Option<String>,
    // server.port
    #[config_default(8080)]
    port: Option<u32>,
}
#[derive(Debug, Configuration)]
#[configuration("datasource")]
struct DatasourceConfig {
    // datasource.host
    #[config_default_t(Some(get_host()))]
    host: Option<String>,
    // datasource.port
    #[config_default(3306)]
    port: Option<u32>,
    // datasource.username
    #[config_default("......")]
    username: Option<String>,
    // datasource.password
    #[config_default("......")]
    password: Option<String>,
    // datasource.database
    #[config_default("test_db")]
    database: Option<String>,
}

fn get_host() -> String {
    "127.0.0.1".to_string()
}
```
#### ApplicationContext 初始化
```rust
struct ApplicationStartEvent;
impl ContextAwareEventListener<ContextInitializingEvent> for ApplicationStartEvent {
    fn on_context_event(&self, _event: &ContextInitializingEvent, context: &ApplicationContext) {
        let container = context.container();
        let datasource_config = container
            .get_component::<DatasourceConfig>()
            .expect("datasource config not found")
            .clone();
        // ... do something with datasource_config
        // context.register_component(Arc::new(datasource_component));
    }
}

fn init_context() -> Arc<ApplicationContext> {
    let context = ApplicationContext::builder()
        // Load a configuration file in YAML, TOML, or Properties format.
        .with_config_file("application.yml")
        .build();
    match context {
        Ok(context) => {
            context.subscribe_to_context_events(ApplicationStartEvent);
            match context.initialize() {
                Ok(_) => Arc::new(context),
                Err(e) => {
                    panic!("failed to initialize context: {}", e);
                }
            }
        }
        Err(e) => panic!("failed to new context: {}", e),
    }
}

fn main() {
    let context = init_context();
    let server_config = context
        .get_component::<ServerConfig>()
        .expect("datasource config not found");
    println!("server config: {:?}", server_config);
    // ... do something with context
    // get more component......
}
```
## IoC / DI

### 初始化容器
推荐使用`ApplicationContext`
```rust
use std::sync::Arc;

fn init_container() {
    let container = ComponentContainer::new();
    match container.initialize() {
        Ok(_) => Arc::new(container),
        Err(e) => panic!("Failed to initialize container {}", e)
    }
}
```

### 注册组件（Component）

#### 自动注册及注入（Derive）

在结构体上添加 `#[derive(Component)]` 宏会自动将一个 `struct` 注册至容器中，默认为单例。对于添加了 `#[autowired]` 的字段会自动从容器中获取实例并进行注入。

```rust
use verdure::Component;

#[derive(Component)]
struct TestA {
    #[autowired]
    test_b: Arc<TestB>,
    test_c: Option<TestC>,
    test_d: TestD
}

#[derive(Component)]
struct TestB {
    a: i32,
    b: i32,
}

struct TestC {
    a: i32
}

#[derive(Default)]
struct TestD {
    a: i32,
}
```

需要注意的事项有二点：

* 被注入的字段必须是 `Arc<T>` 包装
* 对于不需要注入的字段则需要它们是 `Option<T>` 或已实现 `Default` 特性

#### 手动注册及获取组件

```rust
#[derive(Debug)]
struct Config {
    name: &'static str,
    port: u16
}

fn main() {
    let container = init_container();
    container.register_component(Arc::new(config));
    let config = container.get_component::<Config>().unwrap();
    println!("config: {:?}", config);
}
```

### 容器事件监听

#### 使用宏的方式

```rust
fn handle_container_lifecycle(event: &ContainerLifecycleEvent) {
    match event {
        ContainerLifecycleEvent::InitializationStarted {
            container,
            component_count,
        } => {
            // 可以在此处注册初始化时必要的组件
        }
        ContainerLifecycleEvent::InitializationCompleted {
            container: _,
            component_count,
            duration,
        } => {
            println!(
                "容器初始化完成\n组件数量: {}\n耗时: {:?}",
                component_count, duration
            );
        }
        ContainerLifecycleEvent::ComponentCreated {
            container: _,
            component_name,
            component_type_id,
            creation_duration,
        } => {
            println!(
                "组件创建完成:\n名称: {}\n类型ID: {:?}\n创建耗时: {:?}",
                component_name, component_type_id, creation_duration
            );
        }
    }
}
lifecycle_listener!("app_container_listener", handle_container_lifecycle);
```