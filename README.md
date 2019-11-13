# Soccer

## 测试

soccer 端
``` 
cargo run --bin soccer
```

goal 端
```
cargo run --bin goal 
```

客户端

```
export http_proxy=socks5h://localhost:8080
curl -i baidu.com 
```

## 跨平台编译 Linux 版二进制程序

``` 
rumb='docker run --rm -it -v cargo-git:/home/rust/.cargo/git -v cargo-registry:/home/rust/.cargo/registry -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder:1.39.0'
```

```
rumb sh build-scripts/build.sh

ll target/x86_64-unknown-linux-musl/release/
```
