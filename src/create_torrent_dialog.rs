use crate::build_bottom_files;
use crate::create_file_model;
use crate::magnet_tools::magnet_to_metainfo;
use crate::transmission;
use crate::utils::build_tree;
use crate::TorrentCmd;
use base64;
use gtk::prelude::*;
use magnet_url::Magnet;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use tokio::sync::mpsc;
use urlencoding;

pub fn add_torrent_dialog2(
    vbox: &gtk::Box,
    magnet_url: &Option<String>,
    torrent_file: &Option<PathBuf>,
    _cancel_fetch: Rc<Cell<bool>>,
    _magnet_done: Rc<Cell<bool>>,
    magnet_data: Rc<RefCell<Option<Vec<u8>>>>,
    destination: &gtk::DropDown,
    start_paused_checkbox: &gtk::CheckButton,
) {
    use lava_torrent::torrent::v1::Torrent;

    let model = gtk::StringList::new(&vec![
        "/var/lib/transmission/Downloads/films",
        "/var/lib/transmission/Downloads/games",
        "/var/lib/transmission/Downloads/music",
        "/var/lib/transmission/Downloads/lessons",
        "/var/lib/transmission/Downloads/blues",
    ]);
    destination.set_model(Some(&model));

    //  let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);

    let file_hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    vbox.append(&file_hbox);

    let l = gtk::Label::new(Some("Torrent file"));
    file_hbox.append(&l);

    let file_chooser = gtk::Button::new();
    file_hbox.append(&file_chooser);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let l = gtk::Label::new(Some("Destination"));
    hbox.append(&l);
    hbox.append(destination);
    vbox.append(&hbox);

    let file_table = gtk::ColumnView::new(None::<&gtk::NoSelection>);
    let scrolled_files = build_bottom_files(&file_table, false);

    // let stupid_model = create_file_model(&Rc::new(RefCell::new(utils::build_tree(&files))));
    let stupid_model = create_file_model(&Rc::new(RefCell::new(build_tree(&vec![]))));
    let files_selection = gtk::NoSelection::new(Some(&stupid_model));
    file_table.set_model(Some(&files_selection));

    if let Some(ref path_buf) = torrent_file {
        let pb2 = path_buf.clone();
        let path = pb2.into_boxed_path();
        let s = path.to_str().unwrap();
        let torrent = Torrent::read_from_file(path.clone()).unwrap();
        let files = torrent
            .files
            .unwrap()
            .iter()
            .map(|f| transmission::File {
                name: f.path.as_path().to_str().unwrap().to_string(),
                length: f.length as u64,
                bytes_completed: 0,
            })
            .collect();
        let stupid_model = create_file_model(&Rc::new(RefCell::new(build_tree(&files))));
        let files_selection = gtk::NoSelection::new(Some(&stupid_model));
        file_table.set_model(Some(&files_selection));
        file_chooser.set_label(&s);
    }
    let progressbar = gtk::ProgressBar::new();
    let _progressbar = Rc::new(progressbar);
    if let Some(ref magnet) = magnet_url {
        let foo: String = magnet.chars().into_iter().take(60).collect();
        let _file_table = Rc::new(file_table);
        if let Result::Ok(m) = Magnet::new(magnet) {
            let s =
                m.dn.as_ref()
                    .map(|l| urlencoding::decode(l).expect("UTF-8").into_owned());
            //        println!("{}", m.dn.expect("foo"));
            file_chooser.set_label(&s.unwrap_or(foo));
            gtk::glib::MainContext::default()
                .spawn_local(pulse_progress(Rc::clone(&_progressbar), Rc::clone(&_magnet_done)));
            gtk::glib::MainContext::default().spawn_local(fetch_magnet_link(
                magnet.to_string(),
                Rc::clone(&_file_table),
                Rc::clone(&_cancel_fetch),
                Rc::clone(&_magnet_done),
                Rc::clone(&magnet_data),
            ));
        } else {
            // FIXME: report error on wrong magnets..
        }
    }

    scrolled_files.set_min_content_width(580);
    scrolled_files.set_min_content_height(320);
    vbox.append(&scrolled_files);
    vbox.append(&*_progressbar);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    // let l = gtk::Label::new(Some("Torrent priority"));
    // hbox.append(&l);
    //  let priority = gtk::DropDown::builder().build();
    //  hbox.append(&priority);
    vbox.append(&hbox);

    vbox.append(start_paused_checkbox);
    let delete_torrent_file = gtk::CheckButton::builder()
        .active(true)
        .label("Delete torrent file")
        .build();
    vbox.append(&delete_torrent_file);
    delete_torrent_file.set_sensitive(false);
}
pub async fn add_torrent_dialog3(
    response: gtk::ResponseType,
    sender: mpsc::Sender<TorrentCmd>,
    magnet_url: Option<String>,
    torrent_file: Option<PathBuf>,
    filter: Rc<gtk::CustomFilter>,
    _cancel_fetch: Rc<Cell<bool>>,
    _magnet_done: Rc<Cell<bool>>,
    magnet_data: Rc<RefCell<Option<Vec<u8>>>>,
    destination: gtk::DropDown,
    start_paused_checkbox: gtk::CheckButton,
) {
    _cancel_fetch.set(true);
    _magnet_done.set(true);
    if response == gtk::ResponseType::Ok {
        let start_paused = start_paused_checkbox.is_active();
        let folder = destination
            .selected_item()
            .map(|x| x.property_value("string").get::<String>().expect("should be string"));
        if let Some(path_buf) = torrent_file {
            let buf = std::fs::read(path_buf).expect("file invalid");
            let metainfo = base64::encode(buf);
            sender
                .send(TorrentCmd::AddTorrent(folder, None, Some(metainfo), start_paused))
                .await
                .expect("failure snd move");
        } else if let Some(data) = &*magnet_data.borrow() {
            let metainfo = base64::encode(data);
            sender
                .send(TorrentCmd::AddTorrent(folder, None, Some(metainfo), start_paused))
                .await
                .expect("failure snd move");
        } else if let Some(url) = magnet_url {
            sender
                .send(TorrentCmd::AddTorrent(
                    folder,
                    Some(url.to_string()),
                    None,
                    start_paused,
                ))
                .await
                .expect("failure snd move");
        } else {
            println!("Error adding torrent. No magnet link and no torrent file is specified");
        }

        async_std::task::sleep(std::time::Duration::from_millis(3000)).await;
        filter.changed(gtk::FilterChange::LessStrict); // unnecesserry as we adding the torrent
    }
}
pub async fn add_torrent_dialog<W: IsA<gtk::Window>>(
    window: Rc<W>,
    sender: mpsc::Sender<TorrentCmd>,
    magnet_url: Option<String>,
    torrent_file: Option<PathBuf>,
    filter: Rc<gtk::CustomFilter>,
) {
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let _cancel_fetch = Rc::new(Cell::new(false));
    let _magnet_done = Rc::new(Cell::new(false));
    let magnet_data = Rc::new(RefCell::new(None));
    let destination = gtk::DropDown::builder().build();
    let start_paused_checkbox = gtk::CheckButton::builder().active(false).label("Start paused").build();

    let dialog = gtk::Dialog::builder().transient_for(&*window).modal(true).build();

    dialog.content_area().append(&vbox); // and a label for destination))
    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Add", gtk::ResponseType::Ok);

    dialog.set_css_classes(&["simple-dialog"]);
    add_torrent_dialog2(
        &vbox,
        &magnet_url,
        &torrent_file,
        _cancel_fetch.clone(),
        _magnet_done.clone(),
        magnet_data.clone(),
        &destination,
        &start_paused_checkbox,
    );
    let response = dialog.run_future().await;
    dialog.close();
    add_torrent_dialog3(
        response,
        sender,
        magnet_url,
        torrent_file,
        filter,
        _cancel_fetch,
        _magnet_done,
        magnet_data,
        destination,
        start_paused_checkbox,
    )
    .await;
}

async fn pulse_progress(_progressbar: Rc<gtk::ProgressBar>, _magnet_done: Rc<Cell<bool>>) {
    loop {
        if _magnet_done.get() {
            break;
        }
        _progressbar.pulse();
        async_std::task::sleep(std::time::Duration::from_millis(200)).await;
    }
    _progressbar.set_fraction(1.0);
}

async fn fetch_magnet_link(
    uri: String,
    _file_table: Rc<gtk::ColumnView>,
    cancellation: Rc<Cell<bool>>,
    done: Rc<Cell<bool>>,
    magnet_data: Rc<RefCell<Option<Vec<u8>>>>,
) {
    use lava_torrent::torrent::v1::Torrent;
    let maybe_torrent = magnet_to_metainfo(&uri, cancellation).await;
    done.set(true);
    if let Some(ref data) = maybe_torrent {
        let res = Torrent::read_from_bytes(data);
        if let Result::Ok(torrent) = res {
            if let Some(xs) = torrent.files {
                let files = xs
                    .iter()
                    .map(|f| transmission::File {
                        name: f.path.as_path().to_str().expect("bloody name?").to_string(),
                        length: f.length as u64,
                        bytes_completed: 0,
                    })
                    .collect();
                let stupid_model = create_file_model(&Rc::new(RefCell::new(build_tree(&files))));
                let files_selection = gtk::NoSelection::new(Some(&stupid_model));
                _file_table.set_model(Some(&files_selection));
                (*magnet_data).replace(Some(data.to_vec()));
            } else {
                // the file is the name
                // TODO: same needs to be done for file details
                let files = vec![transmission::File {
                    name: torrent.name,
                    length: torrent.length as u64,
                    bytes_completed: 0,
                }];
                let stupid_model = create_file_model(&Rc::new(RefCell::new(build_tree(&files))));
                let files_selection = gtk::NoSelection::new(Some(&stupid_model));
                _file_table.set_model(Some(&files_selection));
                _file_table.set_sensitive(false);
                (*magnet_data).replace(Some(data.to_vec()));
            }
        } else {
            println!("Error: {:?}", res); // FIXME: error mngmnt
        }
    }
}
