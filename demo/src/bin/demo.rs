use std::time::Duration;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// 用户名
    name: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
}

fn main() {
    run();
}

#[tokio::main]
async fn run() {
    ff().await;
}

async fn ff() {
    let t1 = tokio::spawn(async {
        tokio::time::sleep(Duration::from_secs(3)).await;
        println!("1");
    });

    let t2 = tokio::spawn(async {
        tokio::time::sleep(Duration::from_secs(3)).await;
        println!("2");
    });
    let _ = tokio::join!(t1, t2);
    println!("done")
}
