# 网络助手

该项目是一个使用 Vue.js 和 Tauri 构建的应用程序，旨在管理 TCP、UDP 和串口连接。它提供了一个用户友好的界面，用于配置和控制网络通信和串口操作。
![](https://obsidian-1255729190.cos.ap-shanghai.myqcloud.com/20250213221415965.png)

## 功能

- **TCP 客户端和服务器**：连接到 TCP 服务器或建立 TCP 服务器来处理传入连接。
- **UDP 通信**：绑定到 UDP 端口并发送/接收消息。
- **串口管理**：配置和通过串口进行通信，并支持自定义设置。

## 使用技术

- **Vue.js**：用于构建用户界面的前端框架。
- **Element Plus**：Vue.js 的 UI 库，提供预构建的组件。
- **Tauri**：使用 Web 技术构建桌面应用程序的框架，允许访问系统级 API。

## 安装步骤

1. **克隆仓库**：
   ```bash
   git clone https://github.com/yourusername/network-serial-manager.git
   cd network-serial-manager
   ```

2. **安装依赖**：
   ```bash
   npm install
   ```

3. **运行应用程序**：
   ```bash
   npm run tauri dev
   ```

## 使用方法

1. **TCP 客户端**：
   - 输入 IP 地址和端口号。
   - 点击“连接”按钮进行连接或“断开”按钮断开连接。
   - 使用“发送”按钮发送消息。

2. **TCP 服务器**：
   - 设置 IP 地址和端口号。
   - 点击“建立”按钮启动服务器或“断开”按钮停止服务器。
   - 向连接的客户端发送消息。

3. **UDP**：
   - 绑定到本地 IP 和端口。
   - 指定目标 IP 和端口以发送消息。
   - 点击“绑定”进行绑定或“解绑”进行解绑。

4. **串口**：
   - 选择串口并配置波特率、校验位等设置。
   - 点击“打开”按钮打开或“关闭”按钮关闭串口。
   - 通过串口连接发送消息。

## 事件处理

应用程序监听各种事件以实时更新 UI，例如连接状态变化和接收到的消息历史。

## 贡献

欢迎贡献！请 fork 仓库并提交 pull request 以进行任何改进或修复。

## 许可证

此项目根据 MIT 许可证授权。

