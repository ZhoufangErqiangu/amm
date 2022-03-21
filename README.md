# Marco Polo AMM

## 介绍

Marco Polo AMM项目合约程序

## 软件架构

参见https://docs.solana.com/developing/programming-model/overview

## 安装教程

1. 安装rustc v1.56.1

2. 安装solana cli v1.9.5

3. 确认id.json和manager.json文件路径

4. 构建

   ```bash
   cargo build-bpf
   ```

5. 确认id.json文件钱包账户中sol数额充足

6. 部署

   ```bash
   solana program deploy XXXXX.so
   ```

   

## 文件结构

1. /sap_pool/ index基金合约程序
2. /sy_sap/ yield基金合约程序
3. 

## 常用命令

```bash
export PATH="/home/ubuntu/.local/share/solana/install/active_release/bin:$PATH"
solana program deploy sap/sypool.so
solana program deploy sy/sypool.so
```

