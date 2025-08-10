use gtk4::{ScrolledWindow, ListBox};
use gtk4::prelude::*;

pub struct Sidebar {
    pub container: ScrolledWindow,
    pub listbox: ListBox,
}

pub fn build_sidebar() -> Sidebar {
    let container = ScrolledWindow::builder()
        .min_content_width(240)
        .vexpand(true)
        .build();
    let listbox = ListBox::new();
    // Apply Adwaita navigation sidebar styling
    listbox.add_css_class("navigation-sidebar");
    listbox.set_selection_mode(gtk4::SelectionMode::Single);
    container.set_child(Some(&listbox));
    Sidebar { container, listbox }
}
