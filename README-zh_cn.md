# Verdure

[![Crates.io version](https://img.shields.io/crates/v/verdure.svg?style=flat-square)](https://crates.io/crates/verdure)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/verdure)

[English](./README.md) | 简体中文

Verdure - 一个Rust的生态框架

正如它的名字一样，愿景为充满朝气,生气勃勃的一个生态化框架，致力于能提供便捷的Rust开发。

目前处于基础底座功能的打造期，期待有志之士加入一起做这件事，

## 特性

- [x] IOC容器及容器事件监听
- [x] 依赖注入(DI)
- [ ] 自动化配置
- [ ] AOP
- [ ] Context
- [ ] 更多.....

## 引入依赖

```toml
verdure = "0.0.1"
inventory = "0.3"
```
底层目前强依赖于`inventory`，感谢这个优秀的Repo。

## IoC / DI

### 初始化容器

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

#### 自动注册及注入（Drive）

在结构体上添加`Component`宏会自动将一个`struct`注册至容器中，默认为单例，对于添加了`#[autowired]`的字段会自动从容器中获取实例并进行注入。

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

需要注意的事项有二点:

* 被注入的字段必须是`Arc<T>`包装
* 对于不需要注入的字段则需要它们是`Option<T>`或已实现`Default`特性

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
### 使用宏的方式
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