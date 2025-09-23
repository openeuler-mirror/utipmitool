# utipmitool - IPMI管理工具

一个用于控制支持IPMI设备的命令行工具，基于Rust实现。

## 功能特性

- 支持IPMI协议的基本操作
- 命令行界面友好
- 支持RPM包构建
- 跨平台支持（Linux）

## 安装

### 从源码构建

1. 安装Rust工具链：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
## 使用说明
### 基本命令格式：


```bash
utipmitool [子命令] [选项]
```
查看帮助：


```bash
utipmitool --help
```

## 开发

### 依赖
- Rust 1.85+
- cargo
- rpmbuild (用于构建RPM包)
### 代码结构
```shell
src/
├── cli.rs       # 命令行接口
├── ipmi/        # IPMI协议实现
└── interface/   # 底层接口
```
### 贡献
欢迎提交Issue和PR。

## 开源许可证
utipmitool 在 [Apache-2.0](LICENSE)下发布。
