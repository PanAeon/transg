use transg::transmission_client::{Result, TransmissionClient};

#[allow(dead_code)]
#[derive(Debug)]
pub struct MiniTorrent {
    id: i64,
    name: String,
    status: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = TransmissionClient::new("http://192.168.1.217:9091/transmission/rpc");
//    loop {
        let res = client
            .get_torrent_details(vec![1533,1,10,300,100,343,532])
            .await?;

        println!("{:#?}", res);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
 //   }
   Ok(())
}
