# 云海探宝


## 安装


### 使用二进制

- 下载[release](https://github.com/ClassmateLin/upyun/releases), 并解压。


### 自行编译

**请确保您已正确安装了Rust.**

- `git clone https://github.com/ClassmateLin/upyun.git && cd upyun`

- `cargo build --release`, 编译结果在`./target/release`目录下。


## 配置文件

- 填写你的又拍云账号和密码。
  
```toml
[[account]]
username="user1"
password="xxxsafaf"
```

- 如有多个账号, 则配置多个`[[account]]`:
```toml
[[account]]
username="user1"
password="xxxsafaf"


[[account]]
username="user2"
password="xxxsafaf"
...
```

## 执行

`./upyun`



## todo

- [ ] 消息通知