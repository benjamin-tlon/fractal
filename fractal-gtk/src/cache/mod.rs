use gtk;
use gtk::LabelExt;
use std::fs::remove_dir_all;

use failure::err_msg;
use failure::Error;
use std::collections::HashMap;
use types::Room;
use types::RoomList;

use fractal_api::util::cache_path;
use globals;

/* includes for avatar download */
use backend::BKCommand;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;

use std::cell::RefCell;
use std::rc::Rc;
use widgets::AvatarData;

mod state;
pub use self::state::get;
pub use self::state::AppState;
pub use self::state::FCache;

// TODO: remove this struct
#[derive(Serialize, Deserialize)]
pub struct CacheData {
    pub since: Option<String>,
    pub rooms: RoomList,
    pub username: String,
    pub uid: String,
    pub device_id: String,
}

pub fn store(
    rooms: &RoomList,
    since: Option<String>,
    username: String,
    uid: String,
    device_id: String,
) -> Result<(), Error> {
    // don't store all messages in the cache
    let mut cacherooms: Vec<Room> = vec![];
    for r in rooms.values() {
        let mut r = r.clone();
        let skip = match r.messages.len() {
            n if n > globals::CACHE_SIZE => n - globals::CACHE_SIZE,
            _ => 0,
        };
        r.messages = r.messages.iter().skip(skip).cloned().collect();
        // setting prev_batch to none because we're removing some messages so the
        // prev_batch isn't valid now, it's not pointing to the stored last msg
        r.prev_batch = None;
        cacherooms.push(r);
    }

    let st = AppState {
        since,
        username,
        uid,
        device_id,
    };
    get().save_st(st)?;

    // This is slow because we iterate over all room msgs
    // in the future we shouldn't do that, we should remove the
    // Vec<Msg> from the room and treat messages as first level
    // cache objects with something like cache.get_msgs(room),
    // cache.get_msg(room_id, msg_id) and cache.save_msg(msg)
    get().save_rooms(cacherooms)?;

    Ok(())
}

pub fn load() -> Result<CacheData, Error> {
    let st = get().get_st()?;
    let rooms = get().get_rooms()?;
    let mut cacherooms: RoomList = HashMap::new();

    for r in rooms {
        cacherooms.insert(r.id.clone(), r);
    }

    let data = CacheData {
        since: st.since,
        username: st.username,
        uid: st.uid,
        device_id: st.device_id,
        rooms: cacherooms,
    };

    Ok(data)
}

pub fn destroy() -> Result<(), Error> {
    let fname = cache_path("cache.mdl").or(Err(err_msg("Can't remove cache file")))?;
    remove_dir_all(fname).or_else(|_| Err(err_msg("Can't remove cache file")))
}

/// this downloads a avatar and stores it in the cache folder
pub fn download_to_cache(backend: Sender<BKCommand>, name: String, data: Rc<RefCell<AvatarData>>) {
    let (tx, rx) = channel::<(String, String)>();
    let _ = backend.send(BKCommand::GetUserInfoAsync(name.clone(), Some(tx)));

    gtk::timeout_add(50, move || match rx.try_recv() {
        Err(TryRecvError::Empty) => gtk::Continue(true),
        Err(TryRecvError::Disconnected) => gtk::Continue(false),
        Ok(_resp) => {
            data.borrow_mut().redraw_pixbuf();
            gtk::Continue(false)
        }
    });
}

/* Get username based on the MXID, we should cache the username */
pub fn download_to_cache_username(
    backend: Sender<BKCommand>,
    uid: &str,
    label: gtk::Label,
    avatar: Option<Rc<RefCell<AvatarData>>>,
) {
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();
    backend
        .send(BKCommand::GetUserNameAsync(uid.to_string(), tx))
        .unwrap();
    gtk::timeout_add(50, move || match rx.try_recv() {
        Err(TryRecvError::Empty) => gtk::Continue(true),
        Err(TryRecvError::Disconnected) => gtk::Continue(false),
        Ok(username) => {
            label.set_text(&username);
            if let Some(ref rc_data) = avatar {
                let mut data = rc_data.borrow_mut();
                data.redraw_fallback(Some(username));
            }

            gtk::Continue(false)
        }
    });
}

/* Download username for a given MXID and update a emote message
 * FIXME: We should cache this request and do it before we need to display the username in an emote*/
pub fn download_to_cache_username_emote(
    backend: Sender<BKCommand>,
    uid: &str,
    text: &str,
    label: gtk::Label,
    avatar: Option<Rc<RefCell<AvatarData>>>,
) {
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();
    backend
        .send(BKCommand::GetUserNameAsync(uid.to_string(), tx))
        .unwrap();
    let text = text.to_string();
    gtk::timeout_add(50, move || match rx.try_recv() {
        Err(TryRecvError::Empty) => gtk::Continue(true),
        Err(TryRecvError::Disconnected) => gtk::Continue(false),
        Ok(username) => {
            label.set_markup(&format!("<b>{}</b> {}", &username, text));
            if let Some(ref rc_data) = avatar {
                let mut data = rc_data.borrow_mut();
                data.redraw_fallback(Some(username));
            }

            gtk::Continue(false)
        }
    });
}
