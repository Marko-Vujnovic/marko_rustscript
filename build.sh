# rm -rf $HOME/.cargo/.package-cache # Solves: "waiting for file lock on package cache"
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build --release --color always
# RUST_BACKTRACE=1 cargo test --release
[ $? ] && RUST_BACKTRACE=1 cargo run --release
