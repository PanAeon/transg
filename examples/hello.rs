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
    loop {
        let res = client
            .get_torrents(vec![1, 2, 3, 4], vec!["id", "name", "status"])
            .await?;

        let xs = res
            .get("arguments")
            .unwrap()
            .get("torrents")
            .unwrap()
            .as_array()
            .unwrap();
        let ys = xs
            .iter()
            .skip(1)
            .map(|v| MiniTorrent {
                id: v.as_array().unwrap().get(0).unwrap().as_i64().unwrap(),
                name: v
                    .as_array()
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
                status: v.as_array().unwrap().get(2).unwrap().as_i64().unwrap(),
            })
            .collect::<Vec<MiniTorrent>>();
        println!("{:#?}", ys);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
