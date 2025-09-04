# IPMI工具测试文档

## 已实现的子命令列表

### Chassis 命令 (机箱控制)
```
utipmitool chassis status          # 显示机箱状态信息
utipmitool chassis power status    # 显示电源状态
utipmitool chassis power on        # 开机
utipmitool chassis power off       # 关机
utipmitool chassis power cycle     # 电源循环重启
utipmitool chassis power reset     # 硬重启
utipmitool chassis power diag      # 诊断脉冲
utipmitool chassis power soft      # ACPI软关机
utipmitool chassis identify        # 机箱识别灯控制
utipmitool chassis restart-cause   # 显示系统重启原因
utipmitool chassis bootdev         # 设置启动设备
```

### MC 命令 (管理控制器)
```
utipmitool mc info                 # 显示BMC设备信息
utipmitool mc reset warm           # 温重启BMC
utipmitool mc reset cold           # 冷重启BMC
```

### Sensor 命令 (传感器管理)
```
utipmitool sensor list            # 列出所有传感器及其状态
```

### SDR 命令 (传感器数据记录)
```
utipmitool sdr list               # 列出所有SDR记录
utipmitool sdr info               # 显示SDR仓库信息
```

### SEL 命令 (系统事件日志)
```
utipmitool sel info               # 显示SEL信息
utipmitool sel list               # 列出SEL条目
utipmitool sel elist              # 扩展格式列出SEL条目
```

### User 命令 (用户管理)
```
utipmitool user summary           # 显示用户摘要信息
utipmitool user list              # 列出所有用户
utipmitool user set name          # 设置用户名
utipmitool user set password      # 设置用户密码
utipmitool user disable           # 禁用用户
utipmitool user enable            # 启用用户
utipmitool user priv              # 设置用户权限
utipmitool user test              # 测试密码格式
```

### LAN 命令 (网络配置)
```
utipmitool lan print              # 显示LAN配置
utipmitool lan set                # 设置LAN参数
```

## 测试命令列表

### 1. Chassis 命令测试

#### 1.1 Chassis Status 测试 (风险等级: 低)
```bash
# 基本状态查询 - 只读操作，安全
sudo utipmitool chassis status
```

#### 1.2 Chassis Power 测试 (风险等级: 高)
```bash
# 电源状态查询 - 只读操作，安全
sudo utipmitool chassis power status

# 电源控制操作（谨慎使用 - 会影响系统运行）
sudo utipmitool chassis power on      # 开机 - 高风险
sudo utipmitool chassis power off     # 关机 - 高风险，会立即断电
sudo utipmitool chassis power cycle   # 电源循环 - 高风险，会重启系统
sudo utipmitool chassis power reset   # 硬重启 - 高风险，强制重启
sudo utipmitool chassis power diag    # 诊断脉冲 - 中风险
sudo utipmitool chassis power soft    # ACPI软关机 - 高风险，会关闭系统
```

#### 1.3 Chassis Identify 测试 (风险等级: 低)
```bash
# 默认识别（15秒）- 只控制识别灯，安全
sudo utipmitool chassis identify

# 指定时间识别 - 安全
sudo utipmitool chassis identify 10   # 10秒识别
sudo utipmitool chassis identify 30   # 30秒识别
sudo utipmitool chassis identify 255  # 最大时间识别
sudo utipmitool chassis identify 0    # 关闭识别灯
```

#### 1.4 Chassis Restart Cause 测试 (风险等级: 低)
```bash
# 查看重启原因 - 只读操作，安全
sudo utipmitool chassis restart-cause
```

#### 1.5 Chassis Boot Device 测试 (风险等级: 中)
```bash
# 设置启动设备 - 会影响下次启动，中等风险
sudo utipmitool chassis bootdev none     # 无启动设备
sudo utipmitool chassis bootdev pxe      # 设置PXE启动
sudo utipmitool chassis bootdev disk     # 设置硬盘启动
sudo utipmitool chassis bootdev safe     # 设置安全模式启动
sudo utipmitool chassis bootdev diag     # 设置诊断启动
sudo utipmitool chassis bootdev cdrom    # 设置光盘启动
sudo utipmitool chassis bootdev bios     # 设置BIOS启动
sudo utipmitool chassis bootdev floppy   # 设置软盘启动

# 带清除CMOS选项 - 高风险，会重置BIOS设置
sudo utipmitool chassis bootdev disk --clear-cmos
sudo utipmitool chassis bootdev pxe --clear-cmos
```

### 2. MC 命令测试

#### 2.1 MC Info 测试 (风险等级: 低)
```bash
# 显示BMC设备信息 - 只读操作，安全
sudo utipmitool mc info
```

#### 2.2 MC Reset 测试 (风险等级: 高)
```bash
# BMC重启 - 高风险，会重启管理控制器，可能影响远程管理
sudo utipmitool mc reset warm    # 温重启BMC
sudo utipmitool mc reset cold    # 冷重启BMC - 更高风险
```

### 3. Sensor 命令测试 (风险等级: 低)
```bash
# 列出所有传感器 - 只读操作，安全
sudo utipmitool sensor list

# 使用详细输出 - 安全
sudo utipmitool -v sensor list
sudo utipmitool -vv sensor list
```

### 4. SDR 命令测试 (风险等级: 低)
```bash
# 列出SDR记录 - 只读操作，安全
sudo utipmitool sdr list

# 显示SDR仓库信息 - 只读操作，安全
sudo utipmitool sdr info

# 使用详细输出 - 安全
sudo utipmitool -v sdr list
sudo utipmitool -v sdr info
```

### 5. SEL 命令测试 (风险等级: 低)
```bash
# 显示SEL信息 - 只读操作，安全
sudo utipmitool sel info

# 列出SEL条目 - 只读操作，安全
sudo utipmitool sel list

# 扩展格式列出SEL条目 - 只读操作，安全
sudo utipmitool sel elist

# 带参数的SEL列表 - 安全
sudo utipmitool sel list first 10    # 显示前10条
sudo utipmitool sel list last 5      # 显示最后5条
sudo utipmitool sel elist first 10   # 扩展格式显示前10条
```

### 6. User 命令测试 (风险等级: 中-高)
```bash
# 用户信息查询 - 只读操作，安全
sudo utipmitool user summary     # 显示用户摘要
sudo utipmitool user list        # 列出所有用户

# 用户管理操作 - 中到高风险，会影响用户访问
sudo utipmitool user set name 2 testuser        # 设置用户名 - 中风险
sudo utipmitool user set password 2 testpass    # 设置密码 - 高风险
sudo utipmitool user set password 2             # 清除密码 - 高风险
sudo utipmitool user disable 2                  # 禁用用户 - 高风险
sudo utipmitool user enable 2                   # 启用用户 - 中风险
sudo utipmitool user priv 2 4                   # 设置管理员权限 - 高风险
sudo utipmitool user priv 2 2 1                 # 设置用户权限到通道1 - 中风险
sudo utipmitool user test 2 16                  # 测试16字节密码格式 - 低风险
sudo utipmitool user test 2 20 mypass           # 测试20字节密码格式 - 低风险
```

### 7. LAN 命令测试 (风险等级: 中-高)
```bash
# LAN配置查询 - 只读操作，安全
sudo utipmitool lan print           # 显示默认通道配置
sudo utipmitool lan print -c 1      # 显示通道1配置

# LAN配置设置 - 高风险，会影响网络连接
sudo utipmitool lan set -c 1 ipaddr 192.168.1.100    # 设置IP地址 - 高风险
sudo utipmitool lan set -c 1 netmask 255.255.255.0   # 设置子网掩码 - 高风险
sudo utipmitool lan set -c 1 defgw_ipaddr 192.168.1.1 # 设置网关 - 高风险
```

### 8. 全局参数测试 (风险等级: 低)
```bash
# 使用不同接口类型（如果支持）- 安全
sudo utipmitool -I open chassis status

# 使用不同设备号 - 安全
sudo utipmitool -d 0 chassis status
sudo utipmitool -d 1 chassis status

# 详细输出级别 - 安全
sudo utipmitool -v chassis status      # 详细输出
sudo utipmitool -vv chassis status     # 更详细输出
sudo utipmitool -vvv chassis status    # 最详细输出

# CSV输出格式 - 安全
sudo utipmitool -c sensor list         # CSV格式传感器列表
sudo utipmitool -c user summary        # CSV格式用户摘要
```

### 9. 错误处理测试 (风险等级: 低)
```bash
# 测试无效参数 - 安全，只会返回错误信息
sudo utipmitool chassis identify 300      # 超出范围（最大255）
sudo utipmitool chassis bootdev invalid   # 无效启动设备
sudo utipmitool chassis power invalid     # 无效电源操作
sudo utipmitool user priv 2 10           # 无效权限级别

# 测试权限问题（不使用sudo）- 安全
utipmitool chassis status                 # 测试权限不足的情况
utipmitool sensor list                    # 测试权限不足的情况
```

### 10. 帮助信息测试 (风险等级: 无)
```bash
# 主帮助 - 完全安全
utipmitool --help
utipmitool -h

# 子命令帮助 - 完全安全
utipmitool chassis --help
utipmitool chassis power --help
utipmitool chassis identify --help
utipmitool mc --help
utipmitool sensor --help
utipmitool sdr --help
utipmitool sel --help
utipmitool user --help
utipmitool lan --help

# 版本信息 - 完全安全
utipmitool --version
utipmitool -V
```

## 建议的测试顺序

### 第一阶段：安全命令测试（只读操作）
这些命令不会对系统造成任何影响，可以安全地进行测试：

```bash
# 1. 帮助和版本信息（完全安全）
utipmitool --help
utipmitool --version
utipmitool chassis --help
utipmitool mc --help

# 2. 基本状态查询（只读操作）
sudo utipmitool chassis status
sudo utipmitool chassis power status
sudo utipmitool chassis restart-cause
sudo utipmitool mc info

# 3. 传感器和SDR信息（只读操作）
sudo utipmitool sensor list
sudo utipmitool sdr info
sudo utipmitool sdr list

# 4. SEL事件日志查询（只读操作）
sudo utipmitool sel info
sudo utipmitool sel list

# 5. 用户信息查询（只读操作）
sudo utipmitool user summary
sudo utipmitool user list

# 6. LAN配置查询（只读操作）
sudo utipmitool lan print
```

### 第二阶段：低风险命令测试
这些命令有轻微影响但相对安全：

```bash
# 1. 识别灯测试（会让服务器前面板灯闪烁，但不影响系统运行）
sudo utipmitool chassis identify 5    # 5秒识别
sudo utipmitool chassis identify 0    # 关闭识别灯

# 2. 详细输出测试（只是改变输出格式）
sudo utipmitool -v chassis status
sudo utipmitool -vv sensor list
sudo utipmitool -c user summary

# 3. 错误处理测试（测试无效参数）
sudo utipmitool chassis identify 300  # 会返回错误但不影响系统
sudo utipmitool chassis bootdev invalid
```

### 第三阶段：中风险命令测试（谨慎使用）
⚠️ **注意：这些命令会影响系统配置，建议在测试环境中使用**

```bash
# 1. 启动设备设置（会影响下次启动，但不影响当前运行）
sudo utipmitool chassis bootdev pxe   # 设置PXE启动
sudo utipmitool chassis bootdev disk  # 恢复硬盘启动

# 2. 用户管理测试（影响用户访问权限）
sudo utipmitool user set name 3 testuser    # 设置测试用户名
sudo utipmitool user disable 3              # 禁用测试用户
sudo utipmitool user enable 3               # 重新启用用户
sudo utipmitool user priv 3 2               # 设置用户权限

# 3. 密码测试（不会改变实际密码）
sudo utipmitool user test 2 16
sudo utipmitool user test 2 20 testpass
```

### 第四阶段：高风险命令测试（极度谨慎）
🚨 **危险警告：这些命令可能会导致系统重启、关机或失去远程访问，仅在测试环境中使用**

```bash
# 1. 电源控制命令（会直接影响系统运行状态）
# ⚠️ 执行前请确保：
# - 在测试环境中
# - 已保存所有工作
# - 有物理访问权限
# - 做好系统恢复准备

sudo utipmitool chassis power cycle  # 会立即重启系统
sudo utipmitool chassis power reset  # 会强制重启系统
sudo utipmitool chassis power off    # 会关闭系统
sudo utipmitool chassis power soft   # 会发送关机信号

# 2. BMC重启（会影响远程管理功能）
sudo utipmitool mc reset warm        # 温重启BMC
sudo utipmitool mc reset cold        # 冷重启BMC

# 3. 网络配置更改（可能导致失去网络连接）
sudo utipmitool lan set -c 1 ipaddr 192.168.1.100
sudo utipmitool lan set -c 1 netmask 255.255.255.0

# 4. 带CMOS清除的启动设备设置（会重置BIOS设置）
sudo utipmitool chassis bootdev disk --clear-cmos
```

## 风险等级说明

| 风险等级 | 说明 | 影响范围 | 建议 |
|---------|------|----------|------|
| **无** | 完全安全的命令，如帮助信息 | 无任何影响 | 随时可用 |
| **低** | 只读操作，不会改变系统状态 | 仅查询信息 | 生产环境安全 |
| **中** | 会改变配置但不影响当前运行 | 影响下次启动或用户权限 | 测试环境推荐 |
| **高** | 会直接影响系统运行状态 | 可能导致重启、关机或失去连接 | 仅测试环境，需备份 |

## 测试注意事项

### 权限要求
- 大部分IPMI命令需要root权限
- 确保用户在`ipmi`组中或使用`sudo`
- 某些命令可能需要特定的IPMI权限级别

### 硬件要求
- 需要支持IPMI的硬件平台
- 确保IPMI模块已加载：`lsmod | grep ipmi`
- 确保设备文件存在：`ls -la /dev/ipmi*`
- 检查IPMI服务状态：`systemctl status ipmi`

### 安全建议
1. **在生产环境中测试时要格外小心**
2. **电源控制命令可能导致系统重启或关机**
3. **BMC重启会影响远程管理功能**
4. **网络配置更改可能导致失去网络连接**
5. **用户管理命令会影响IPMI访问权限**
6. **启动设备设置会影响下次系统启动**
7. **建议先在测试环境中验证所有功能**

### 故障排除
如果命令执行失败，检查以下项目：

1. **IPMI模块加载**：
   ```bash
   sudo modprobe ipmi_devintf
   sudo modprobe ipmi_si
   ```

2. **设备文件权限**：
   ```bash
   ls -la /dev/ipmi*
   sudo chmod 666 /dev/ipmi0
   ```

3. **系统日志**：
   ```bash
   dmesg | grep -i ipmi
   journalctl -u ipmi
   ```

## 预期输出示例

### Chassis Status 输出
```
System Power         : on
Power Overload       : false
Power Interlock      : inactive
Main Power Fault     : false
Power Control Fault  : false
Power Restore Policy : always-off
Last Power Event     : 
Chassis Intrusion    : inactive
Front-Panel Lockout  : inactive
Drive Fault          : false
Cooling/Fan Fault    : false
```

### Sensor List 输出
```
CPU Temp         | 45.000     | degrees C  | ok    | na        | na        | na        | 85.000    | 90.000    | na        
System Temp      | 28.000     | degrees C  | ok    | na        | na        | na        | 80.000    | 85.000    | na        
Fan1             | 2100.000   | RPM        | ok    | na        | 500.000   | na        | na        | na        | na        
```

### Power Status 输出
```
Chassis Power is on
```

## 性能基准

与C版本ipmitool的对比：
- **Sensor数量**：Rust版本46个 vs C版本59个
- **功能完整性**：Chassis命令100%兼容
- **响应时间**：基本相当
- **内存使用**：Rust版本更安全，无内存泄漏风险
