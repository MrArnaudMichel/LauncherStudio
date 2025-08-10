use gtk4::{Box as GtkBox, Label, Orientation};
use gtk4::prelude::*;

pub struct StatusBar {
    pub container: GtkBox,
    pub label: Label,
}

pub fn build_status_bar() -> StatusBar {
    let container = GtkBox::new(Orientation::Horizontal, 6);
    let label = Label::new(Some("No file selected"));
    label.set_xalign(0.0);
    container.append(&label);
    StatusBar { container, label }
}
