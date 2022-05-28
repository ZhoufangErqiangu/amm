# 自动做市机

## 总览

自动做市机智能合约

测试页面: https://www.aposimz.xyz/amm/ (只可链接到devnet)

**此代码未经审计,使用者自行承担风险**

## 软件架构

参见 https://docs.solana.com/developing/programming-model/overview

## 设计

### 角色

1. 拥有者
2. 用户

### 业务操作

1. 创建

   拥有者创建amm pool,创建时转入一定数量的两种token

   由此产生固定的k,k值为转入的token数额之积

2. swap

   用户转入一种token,换取另外一种token

   转入转出的数额满足以下公式
   $$
   (a-∆a)*(b+∆b)=k
   $$
   实际由于计算精度有限,会存在一定误差

3. 终止

   拥有者终止amm pool,提取其中所有的token,并关闭所有账户

4. fee

   用户swap时向amm pool中额外转入一定数额的token作为fee

   fee的mint和倍率由拥有者在创建时指定

   拥有者可随时提取fee

   终止时,拥有者提取所有fee

## 安装

1. 安装 rustc v1.56.1,参见https://www.rust-lang.org/tools/install

2. 安装 solana cli v1.9.5,参见https://docs.solana.com/cli/install-solana-cli-tools

3. 确认 id.json 文件路径

4. build

   ```bash
   cargo build-bpf
   ```

5. 确认 id.json 文件钱包中拥有足够的SOL

6. 部署

   ```bash
   solana program deploy target/deploy/amm.so
   ```

7. 编辑js/index.js中的AmmProgramId

7. 测试

   ```bash
   npm run test
   ```

## 文件

1. src/ 合约代码
2. js/ 调用合约的js代码
3. client/ 客户端应用

## 已知问题

1. 当某一token数额为0,或将变为0时,swap失败
2. fee以amm pool中的一种token数额为基础计算,可能由于精度问题无法计算
3. 尚无合理方法检查swap误差

## 计划

超级swap

假设有两个amm pool,其token分别为a/b和b/c,则进行两次swap,以b为中间量,直接swap a/c.

## 常用命令

```bash
export PATH="/home/ubuntu/.local/share/solana/install/active_release/bin:$PATH"
solana program deploy amm.so
```

