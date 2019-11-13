
cat <<HERE >> /home/rust/.cargo/config

[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"
HERE
