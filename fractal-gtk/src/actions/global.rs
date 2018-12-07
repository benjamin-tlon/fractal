use std::sync::{Arc, Mutex};

use appop::AppOp;
use appop::AppState;
use gio::prelude::*;
use gio::SimpleAction;
use gtk::prelude::*;

/* This creates globale actions which are connected to the application */
/* TODO: Remove op */
pub fn new(op: Arc<Mutex<AppOp>>) {
    let app = op.lock().unwrap().gtk_app.clone();
    let settings = SimpleAction::new("settings", None);
    let account = SimpleAction::new("account_settings", None);
    let dir = SimpleAction::new("directory", None);
    let chat = SimpleAction::new("start_chat", None);
    let newr = SimpleAction::new("new_room", None);
    let joinr = SimpleAction::new("join_room", None);
    let logout = SimpleAction::new("logout", None);

    let room = SimpleAction::new("room_details", None);
    let inv = SimpleAction::new("room_invite", None);
    let search = SimpleAction::new("search", None);
    let leave = SimpleAction::new("leave_room", None);

    let quit = SimpleAction::new("quit", None);
    let shortcuts = SimpleAction::new("shortcuts", None);
    let about = SimpleAction::new("about", None);

    let open_room = SimpleAction::new("open_room", glib::VariantTy::new("s").ok());

    app.add_action(&settings);
    app.add_action(&account);
    app.add_action(&dir);
    app.add_action(&chat);
    app.add_action(&newr);
    app.add_action(&joinr);
    app.add_action(&logout);

    app.add_action(&room);
    app.add_action(&inv);
    app.add_action(&search);
    app.add_action(&leave);

    app.add_action(&quit);
    app.add_action(&shortcuts);
    app.add_action(&about);
    app.add_action(&open_room);

    quit.connect_activate(clone!(op => move |_, _| op.lock().unwrap().quit() ));
    about.connect_activate(clone!(op => move |_, _| op.lock().unwrap().about_dialog() ));

    settings.connect_activate(move |_, _| {
        info!("SETTINGS");
    });
    settings.set_enabled(false);

    account.connect_activate(
        clone!(op => move |_, _| op.lock().unwrap().show_account_settings_dialog()),
    );

    dir.connect_activate(
        clone!(op => move |_, _| op.lock().unwrap().set_state(AppState::Directory) ),
    );
    logout.connect_activate(clone!(op => move |_, _| op.lock().unwrap().logout() ));
    room.connect_activate(clone!(op => move |_, _| op.lock().unwrap().show_room_settings() ));
    inv.connect_activate(clone!(op => move |_, _| op.lock().unwrap().show_invite_user_dialog() ));
    chat.connect_activate(clone!(op => move |_, _| op.lock().unwrap().show_direct_chat_dialog() ));
    leave.connect_activate(clone!(op => move |_, _| op.lock().unwrap().leave_active_room() ));
    newr.connect_activate(clone!(op => move |_, _| op.lock().unwrap().new_room_dialog() ));
    joinr.connect_activate(clone!(op => move |_, _| op.lock().unwrap().join_to_room_dialog() ));

    open_room.connect_activate(clone!(op => move |_, data| {
        if let Some(id) = get_room_id(data) {
                op.lock().unwrap().set_active_room_by_id(id.to_string());
        }
    }));

    /* Add Keybindings to actions */
    app.set_accels_for_action("app.quit", &["<Ctrl>Q"]);
}

fn get_room_id(data: &Option<glib::Variant>) -> Option<&str> {
    data.as_ref()?.get_str()
}