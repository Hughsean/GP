use clap::{Parser, Subcommand};

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// 自己的用户名
    #[arg(short, long)]
    pub name: String,
    /// 服务器地址
    #[arg(short, long)]
    pub addr: String,
    /// 服务器名称
    #[arg(short, long)]
    pub server: String,
    /// 证书路径
    #[arg(short, long)]
    pub cert: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// 在服务器上挂机等待
    Wait,
    /// 呼叫用户
    Call {
        /// 被呼叫用户名
        #[arg(short, long)]
        name: String,
    },
    /// 查询服务器可呼叫用户
    Query,
}
