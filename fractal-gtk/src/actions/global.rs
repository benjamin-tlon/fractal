use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use appop::AppOp;
use fractal_api::types::Message;
use gio::prelude::*;
use gio::SimpleAction;
use glib;
use gtk::prelude::*;
use App;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Login,
    Loading,
    NoRoom,
    Room,
    RoomSettings,
    MediaViewer,
    AccountSettings,
    Directory,
}

impl<'a> From<&'a glib::Variant> for AppState {
    fn from(v: &glib::Variant) -> AppState {
        v.get::<String>().expect("Invalid back state type").into()
    }
}

impl From<String> for AppState {
    fn from(v: String) -> AppState {
        match v.as_str() {
            "login" => AppState::Login,
            "loading" => AppState::Loading,
            "no-room" => AppState::NoRoom,
            "room" => AppState::Room,
            "media-viewer" => AppState::MediaViewer,
            "account-settings" => AppState::AccountSettings,
            "room-settings" => AppState::RoomSettings,
            "directory" => AppState::Directory,
            _ => panic!("Invalid back state type"),
        }
    }
}

impl From<AppState> for glib::Variant {
    fn from(v: AppState) -> glib::Variant {
        match v {
            AppState::Login => "login".to_variant(),
            AppState::Loading => "loading".to_variant(),
            AppState::NoRoom => "no-room".to_variant(),
            AppState::Room => "room".to_variant(),
            AppState::MediaViewer => "media-viewer".to_variant(),
            AppState::AccountSettings => "account-settings".to_variant(),
            AppState::RoomSettings => "room-setting".to_variant(),
            AppState::Directory => "directory".to_variant(),
        }
    }
}

/* This creates globale actions which are connected to the application */
/* TODO: Remove op */
pub fn new(app: &gtk::Application, op: &Arc<Mutex<AppOp>>) {
    let settings = SimpleAction::new("settings", None);
    let chat = SimpleAction::new("start_chat", None);
    let newr = SimpleAction::new("new_room", None);
    let joinr = SimpleAction::new("join_room", None);
    let logout = SimpleAction::new("logout", None);

    let inv = SimpleAction::new("room_invite", None);
    let search = SimpleAction::new("search", None);
    let leave = SimpleAction::new("leave_room", None);

    let shortcuts = SimpleAction::new("shortcuts", None);
    let about = SimpleAction::new("about", None);
    let quit = gio::SimpleAction::new("quit", None);

    let open_room = SimpleAction::new("open-room", glib::VariantTy::new("s").ok());
    let back = SimpleAction::new("back", None);
    let media_viewer = SimpleAction::new("open-media-viewer", glib::VariantTy::new("s").ok());
    let account = SimpleAction::new("open-account-settings", None);
    let directory = SimpleAction::new("directory", None);
    //TODO: use roomid as value
    let room_settings = SimpleAction::new("open-room-settings", None);

    app.add_action(&settings);
    app.add_action(&account);
    app.add_action(&chat);
    app.add_action(&newr);
    app.add_action(&joinr);
    app.add_action(&logout);

    app.add_action(&inv);
    app.add_action(&search);
    app.add_action(&leave);

    app.add_action(&quit);
    app.add_action(&shortcuts);
    app.add_action(&about);
    app.add_action(&open_room);
    app.add_action(&back);
    app.add_action(&directory);
    app.add_action(&room_settings);
    app.add_action(&media_viewer);
    app.add_action(&account);

    // When activated, shuts down the application
    let app_weak = app.downgrade();
    quit.connect_activate(move |_action, _parameter| {
        let app = upgrade_weak!(app_weak);
        app.quit();
    });

    about.connect_activate(clone!(op => move |_, _| op.lock().unwrap().about_dialog() ));

    settings.connect_activate(move |_, _| {
        info!("SETTINGS");
    });
    settings.set_enabled(false);

    logout.connect_activate(clone!(op => move |_, _| op.lock().unwrap().logout() ));
    inv.connect_activate(clone!(op => move |_, _| op.lock().unwrap().show_invite_user_dialog() ));
    chat.connect_activate(clone!(op => move |_, _| op.lock().unwrap().show_direct_chat_dialog() ));
    leave.connect_activate(clone!(op => move |_, _| op.lock().unwrap().leave_active_room() ));
    newr.connect_activate(clone!(op => move |_, _| op.lock().unwrap().new_room_dialog() ));
    joinr.connect_activate(clone!(op => move |_, _| op.lock().unwrap().join_to_room_dialog() ));

    /* Store the history of views so we can go back to it, this will be kept alive by the back
     * callback */
    let back_history: Rc<RefCell<Vec<AppState>>> = Rc::new(RefCell::new(vec![]));

    let back_weak = Rc::downgrade(&back_history);
    account.connect_activate(clone!(op => move |_, _| {
        op.lock().unwrap().show_account_settings_dialog();

        let back = upgrade_weak!(back_weak);
        back.borrow_mut().push(AppState::AccountSettings);
    }));

    let back_weak = Rc::downgrade(&back_history);
    directory.connect_activate(clone!(op => move |_, _| {
        op.lock().unwrap().set_state(AppState::Directory);

    let back = upgrade_weak!(back_weak);
    back.borrow_mut().push(AppState::Directory);
    }));

    /* TODO: We could pass a message to this to highlight it in the room history, might be
     * handy when opening the room from a notification */
    let back_weak = Rc::downgrade(&back_history);
    open_room.connect_activate(clone!(op => move |_, data| {
        if let Some(id) = get_room_id(data) {
            op.lock().unwrap().set_active_room_by_id(id.to_string());
            /* This does nothing if fractal is already in focus */
            op.lock().unwrap().activate();
        }
        let back = upgrade_weak!(back_weak);
        // Push a new state only if the current state is not already Room
        let push = if let Some(last) = back.borrow().last() {
            last != &AppState::Room
        } else {
            true
        };
        if push {
            back.borrow_mut().push(AppState::Room);
        }
    }));

    let back_weak = Rc::downgrade(&back_history);
    room_settings.connect_activate(clone!(op => move |_, _| {
        op.lock().unwrap().create_room_settings();

        let back = upgrade_weak!(back_weak);
        back.borrow_mut().push(AppState::RoomSettings);
    }));

    let back_weak = Rc::downgrade(&back_history);
    media_viewer.connect_activate(move |_, data| {
        open_viewer(data);

        let back = upgrade_weak!(back_weak);
        back.borrow_mut().push(AppState::MediaViewer);
    });

    // back_history is moved into this closure to keep it alive as long the action exists
    back.connect_activate(move |_, _| {
        // Remove the current state form the store
        back_history.borrow_mut().pop();
        if let Some(state) = back_history.borrow().last() {
            debug!("Go back to state {:?}", state);
            if let Some(op) = App::get_op() {
                let mut op = op.lock().unwrap();
                op.set_state(state.clone());
            }
        } else {
            // Falback when there is no back history
            debug!("There is no state to go back to. Go back to state NoRoom");
            if let Some(op) = App::get_op() {
                let mut op = op.lock().unwrap();
                if op.logged_in {
                    op.set_state(AppState::NoRoom);
                }
            }
        }
    });

    /* Add Keybindings to actions */
    app.set_accels_for_action("app.quit", &["<Ctrl>Q"]);
    app.set_accels_for_action("app.back", &["Escape"]);

    // TODO: Mark active room as read when window gets focus
    //op.lock().unwrap().mark_active_room_messages();
}

fn get_room_id(data: &Option<glib::Variant>) -> Option<&str> {
    data.as_ref()?.get_str()
}

fn get_message(data: &Option<glib::Variant>) -> Option<Message> {
    get_message_by_id(data.as_ref()?.get_str()?)
}

/* TODO: get message from stroage once implemented */
fn get_message_by_id(id: &str) -> Option<Message> {
    let op = App::get_op()?;
    let op = op.lock().unwrap();
    let room_id = op.active_room.as_ref()?;
    op.get_message_by_id(room_id, id)
}

fn open_viewer(data: &Option<glib::Variant>) -> Option<()> {
    let msg = get_message(data)?;
    let op = App::get_op()?;
    let mut op = op.lock().unwrap();
    op.create_media_viewer(msg);
    None
}
