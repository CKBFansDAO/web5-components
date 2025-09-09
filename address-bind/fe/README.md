# Address Bind Frontend

这是一个用于生成 CKB 地址绑定信息的命令行工具。

## 安装

```bash
npm install
```

## 构建

```bash
npm run build
```

## 使用方法

```bash
npm start <fromAddress> <toAddress>
```

### 参数说明

- `fromAddress`: CKB 源地址
- `toAddress`: CKB 目标地址

### 示例

```bash
npm start ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq2jk6pyw9vlnfakx7vp4t5lxg0lzvvsp3c5adflu ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq2jk6pyw9vlnfakx7vp4t5lxg0lzvvsp3c5adflu
```

## 输出

程序将输出生成的 BindInfo 对象及其序列化后的数据。