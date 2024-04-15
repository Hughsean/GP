
fn main() {
    std::env::set_var("RUST_LOG", "DEBUG");
    // 只有注册 subscriber 后， 才能在控制台上看到日志输出

    // let foo = 42;
    // // 调用 `tracing` 包的 `info!`
    // tracing::info!(foo, "Hello from tracing");
}
