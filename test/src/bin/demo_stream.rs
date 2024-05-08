use std::time::Duration;

use common::endpoint_config::{make_endpoint, EndpointType};

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let r = rt.block_on(fun());

    println!("{r:?}")
}

async fn fun() -> anyhow::Result<()> {
    let sendp = make_endpoint(EndpointType::Server("0.0.0.0:12345".parse()?))?;
    let cendpa = make_endpoint(EndpointType::Client("0.0.0.0:0".parse()?))?;

    let t = tokio::spawn(async move {
        let conn = sendp.accept().await.ok_or(anyhow::anyhow!(""))?.await?;
        let mut s = conn.open_uni().await?;
        s.write_all(b"nihao").await?;
        s.finish().await?;
        anyhow::Ok(())
    });

    let c = cendpa
        .connect("127.0.0.1:12345".parse()?, "localhost")?
        .await?;
    let mut r = c.accept_uni().await?;
    tokio::time::sleep(Duration::from_millis(900)).await;

    let data = r.read_to_end(usize::MAX).await?;

    println!("{}", String::from_utf8(data)?);
    t.await??;
    Ok(())
}
