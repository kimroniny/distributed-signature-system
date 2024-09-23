
# 分布式签名系统

这是一个分布式签名系统的实现，包含多个节点的签名服务、公钥收集服务和客户端。

## 项目结构

- `src/main.rs` - 主节点服务，处理消息接收和签名聚合。
- `src/key_collector.rs` - 公钥收集服务，接收和存储公钥。
- `src/bin/client.rs` - 客户端，发送消息、检查状态和验证签名。

## 依赖项

确保你已经安装了以下依赖项：

- Rust (建议使用最新的稳定版本)
- Cargo (Rust 的包管理工具)

## 执行命令

### 1. 运行公钥收集服务

在一个终端中，运行以下命令以启动公钥收集服务：

```bash
cargo run --bin key_collector -- --addr 127.0.0.1:8070
```

公钥收集服务将监听在 `127.0.0.1:8070`。

### 2. 运行主节点服务

在另一个终端中，运行以下命令以启动主节点服务：

```bash
cargo run -- --node-addr 127.0.0.1:8080 --web-addr 127.0.0.1:8090 --key-collector 127.0.0.1:8070 --other-nodes 127.0.0.1:8081 --other-nodes 127.0.0.1:8082
cargo run -- --node-addr 127.0.0.1:8081 --web-addr 127.0.0.1:8091 --key-collector 127.0.0.1:8070 --other-nodes 127.0.0.1:8080 --other-nodes 127.0.0.1:8082
cargo run -- --node-addr 127.0.0.1:8082 --web-addr 127.0.0.1:8092 --key-collector 127.0.0.1:8070 --other-nodes 127.0.0.1:8080 --other-nodes 127.0.0.1:8081
```

主节点服务将监听在 `127.0.0.1:8080`，并提供 Web 服务在 `127.0.0.1:8090`。

## 使用说明

### 发送消息

在客户端中，使用以下命令发送消息：

```bash
cargo run --bin client -- --server http://127.0.0.1:8090 --key-collector http://127.0.0.1:8070 send --message "你的消息"
```

### 检查状态

在客户端中，使用以下命令检查状态：

```bash
cargo run --bin client -- --server http://127.0.0.1:8090 --key-collector http://127.0.0.1:8070 check --request-id <请求ID>
```

### 验证签名

在客户端中，使用以下命令验证签名：

```bash
cargo run --bin client -- --server http://127.0.0.1:8090 --key-collector http://127.0.0.1:8070 verify --request-id <请求ID> --message "你的消息"
```

## 注意事项

- 确保所有服务都在不同的终端中运行。
- 根据需要调整 IP 地址和端口号。
- 确保在发送公钥时，节点能够通过 HTTP POST 请求将公钥发送到 `http://127.0.0.1:8070/receive_key`。

## 贡献

欢迎任何形式的贡献！请提交问题或拉取请求。

## 许可证

此项目采用 MIT 许可证，详细信息请查看 LICENSE 文件。