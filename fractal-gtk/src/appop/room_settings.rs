use gtk;
use gtk::prelude::*;

use actions::AppState;
use appop::AppOp;

use widgets;

impl AppOp {
    pub fn create_room_settings(&mut self) -> Option<()> {
        let stack = self
            .ui
            .builder
            .get_object::<gtk::Stack>("main_content_stack")
            .expect("Can't find main_content_stack in ui file.");
        let stack_header = self
            .ui
            .builder
            .get_object::<gtk::Stack>("headerbar_stack")
            .expect("Can't find headerbar_stack in ui file.");

        {
            let room = self.rooms.get(&self.active_room.clone()?)?;
            let mut panel =
                widgets::RoomSettings::new(self.backend.clone(), self.uid.clone(), room.clone());
            let (body, header) = panel.create()?;

            /* remove old panel */
            if let Some(widget) = stack.get_child_by_name("room-settings") {
                stack.remove(&widget);
            }
            if let Some(widget) = stack_header.get_child_by_name("room-settings") {
                stack_header.remove(&widget);
            }

            stack.add_named(&body, "room-settings");
            stack_header.add_named(&header, "room-settings");

            self.room_settings = Some(panel);
        }

        self.set_state(AppState::RoomSettings);

        None
    }

    pub fn show_new_room_avatar(&self) -> Option<()> {
        let panel = self.room_settings.clone()?;
        panel.show_new_room_avatar();
        None
    }

    pub fn show_new_room_name(&self) -> Option<()> {
        let panel = self.room_settings.clone()?;
        panel.show_new_room_name();
        None
    }

    pub fn show_new_room_topic(&self) -> Option<()> {
        let panel = self.room_settings.clone()?;
        panel.show_new_room_topic();
        None
    }
}
