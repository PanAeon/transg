use lazy_static::lazy_static;
use crate::transmission::{SessionStats, TransmissionClient, FreeSpace, TorrentDetails, TorrentAdd};
use tokio::sync::mpsc::{Sender, Receiver, channel};
use crate::utils::build_tree;
use tokio::runtime::Runtime;
use tokio::time::sleep;
use std::time::Duration;
use std::fs;

        #[derive(Debug)]
        pub enum TorrentUpdate {
          Full(serde_json::Value),
          Partial(serde_json::Value, serde_json::Value, u64),
          Stats(SessionStats),
          FreeSpace(FreeSpace),
          Details(TorrentDetails)
        }

        #[derive(Debug)]
        pub enum TorrentCmd {
            Tick(u64),
            OpenDlDir(i64),
            OpenDlTerm(i64),
            GetDetails(i64),
            QueueMoveUp(Vec<i64>),
            QueueMoveDown(Vec<i64>),
            QueueMoveTop(Vec<i64>),
            QueueMoveBottom(Vec<i64>),
            Delete(Vec<i64>, bool),
            Start(Vec<i64>),
            StartNow(Vec<i64>),
            Stop(Vec<i64>),
            Verify(Vec<i64>),
            Reannounce(Vec<i64>),
            Move(Vec<i64>, String, bool),
            AddTorrent(Option<String>, Option<String>, Option<String>, bool),// download dir, filename, metainfo, start_paused
            PoisonPill()
        }

lazy_static!{
  pub static ref TORRENT_INFO_FIELDS: Vec<&'static str> = vec!["id", "name", "status", "percentDone", "error", "errorString", 
                "eta", "queuePosition", "isFinished", "isStalled", "metadataPercentComplete",
                "peersConnected", "rateDownload", "rateUpload", "recheckProgress",
                "sizeWhenDone", "downloadDir", "uploadedEver", "uploadRatio", "addedDate"];
}

struct CommandProcessor {
  sender : Sender<TorrentCmd>,
  receiver : Option<Receiver<TorrentCmd>>,
  update_sender: Sender<TorrentUpdate>,
  // update_receiver: Receiver<TorrentUpdate>
}

pub fn create() {
   //let (tx2, rx2) = channel(); 
}

impl CommandProcessor {

  pub fn stop(&self) {
     self.sender.blocking_send(TorrentCmd::PoisonPill()).expect("can't stop..");
  }
    // http://192.168.1.217:9091/transmission/rpc
  pub fn run(&mut self, transmission_url: &'static str, rt:Runtime) {
        let sender = self.sender.clone();

        let update_sender = self.update_sender.clone();
        let _receiver = std::mem::replace(&mut self.receiver, None);
        let mut receiver = _receiver.unwrap();

          let mut i:u64 = 0;
          rt.spawn(async move {      
              loop {
                  sleep(Duration::from_secs(2)).await;
                  let res = sender.send(TorrentCmd::Tick(i)).await;
                  if res.is_err() {
                      return;
                  }
                  i += 1;
              }
          });

          rt.spawn(async move {
              let client = TransmissionClient::new(transmission_url);
              let response = client.get_all_torrents(&TORRENT_INFO_FIELDS).await.expect("oops1");
              let ts = response.get("arguments").unwrap().get("torrents").unwrap().to_owned(); 
              update_sender.send(TorrentUpdate::Full(ts)).await.expect("blah");
              loop { // should move into async
            let cmd = receiver.recv().await.expect("probably ticker thread panicked");

            match cmd {
                TorrentCmd::GetDetails(id) => {
                  let details = client.get_torrent_details(vec![id]).await.expect("oops3"); // TODO: what if id is wrong?
                  if details.arguments.torrents.len() > 0 {
                      let res = update_sender.send(TorrentUpdate::Details(details.arguments.torrents[0].to_owned())).await;
                      if res.is_err() {
                          println!("{:#?}", res.err().unwrap());
                      }
                  }                                                                                            
                },
                TorrentCmd::Tick(i) => {
                let foo = client.get_recent_torrents(&TORRENT_INFO_FIELDS).await.expect("oops2");
                let torrents = foo.get("arguments").unwrap().get("torrents").unwrap().to_owned(); 
                let removed  = foo.get("arguments").unwrap().get("removed").unwrap().to_owned(); 
//                println!("Received {} torrents", torrents.as_array().unwrap().len());
                //let num_torrents = torrents.as_array().unwrap().len();
                //if num_torrents < 100 { 
                  update_sender.send(TorrentUpdate::Partial(torrents, removed, i)).await.expect("blah");
                //}

                if i % 3 == 0 {
                  let stats = client.get_session_stats().await.expect("boo");
                  update_sender.send(TorrentUpdate::Stats(stats.arguments)).await.expect("foo");
                }
                if i % 60 == 0 {
                  let free_space = client.get_free_space("/var/lib/transmission/Downloads").await.expect("brkjf");
                  update_sender.send(TorrentUpdate::FreeSpace(free_space.arguments)).await.expect("foo");
                }
                },
                TorrentCmd::OpenDlDir(id) => {
                    let details = client.get_torrent_details(vec![id]).await.expect("oops3"); // TODO: what if id is wrong?
                    if details.arguments.torrents.len() > 0 {
                      let location = details.arguments.torrents[0].download_dir.clone();
                      let my_loc   = location.replace("/var/lib/transmission/Downloads", "/run/mount/transmission/Downloads");
                      let me_loc2  = my_loc.clone();
                      let tree = build_tree(&details.arguments.torrents[0].files);
                      let p = my_loc + "/" + &tree[0].path;  
                      if tree.len() == 1 && fs::read_dir(&p).is_ok() {
                        std::process::Command::new("/home/vitalii/.nix-profile/bin/nautilus")
                         .arg(p)
                         .spawn()
                         .expect("failed to spawn");
                      } else {
                        std::process::Command::new("/home/vitalii/.nix-profile/bin/nautilus")
                         .arg(me_loc2)
                         .spawn()
                         .expect("failed to spawn");

                      }
                    }
                },
                TorrentCmd::OpenDlTerm(id) => { // TODO: refactor both into single function
                    let details = client.get_torrent_details(vec![id]).await.expect("oops3"); // TODO: what if id is wrong?
                    if details.arguments.torrents.len() > 0 {
                      let location = details.arguments.torrents[0].download_dir.clone();
                      let my_loc   = location.replace("/var/lib/transmission/Downloads", "/run/mount/transmission/Downloads");
                      let me_loc2  = my_loc.clone();
                      let tree = build_tree(&details.arguments.torrents[0].files);
                      let p = my_loc + "/" + &tree[0].path;  
                      if tree.len() == 1 && fs::read_dir(&p).is_ok() {
                        std::process::Command::new("/home/vitalii/.nix-profile/bin/alacritty")
                         .arg("--working-directory")
                         .arg(&p)
                         .spawn()
                         .expect("failed to spawn");
                      } else {
                        std::process::Command::new("/home/vitalii/.nix-profile/bin/alacritty")
                         .arg("--working-directory")
                         .arg(&me_loc2)
                         .spawn()
                         .expect("failed to spawn");

                      }
                    }
                },
                TorrentCmd::QueueMoveUp(ids) => {
                    client.queue_move_up(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::QueueMoveDown(ids) => {
                    client.queue_move_down(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::QueueMoveTop(ids) => {
                    client.queue_move_top(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::QueueMoveBottom(ids) => {
                    client.queue_move_bottom(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::Delete(ids, delete_local_data) => {
                    client.torrent_remove(ids, delete_local_data).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::Start(ids) => {
                    client.torrent_start(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::StartNow(ids) => {
                    client.torrent_start_now(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::Stop(ids) => {
                    client.torrent_stop(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::Verify(ids) => {
                    client.torrent_verify(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::Reannounce(ids) => {
                    client.torrent_reannounce(ids).await.expect("oops3"); // TODO: proper error handling 
                },
                TorrentCmd::Move(ids, location, is_move) => {
                    client.torrent_move(ids, &location, is_move).await.expect("ooph4");
                },
                TorrentCmd::AddTorrent(download_dir, filename, metainfo, paused) => {
                    let tadd = TorrentAdd {
                        cookies: None,
                        bandwith_priority: None,
                        download_dir,
                        filename,
                        metainfo,
                        files_unwanted: None,
                        files_wanted: None,
                        labels: None,
                        paused: Some(paused),
                        peer_limit: None,
                        priority_high: None,
                        priority_low: None,
                        priority_normal: None
                    };
                    println!("adding torrent");
                    let res = client.torrent_add(&tadd).await.expect("ooph5");
                    let result = res.as_object().expect("should return object").get("result").expect("must result").as_str().unwrap().to_string();
                    if result == "success" {
                       //  TODO: use gtk notifications
                    //    let _ =  notify("Torrent Added!", "").await; // TODO: add name
                    } else {
                     //   let _ = notify("Error!", "").await;
                      println!("{:?}", res);
                    }
                    println!("{:?}", res);
                },
            TorrentCmd::PoisonPill() => {
            }
            }

                  
          }
          });


          
   
}
}
