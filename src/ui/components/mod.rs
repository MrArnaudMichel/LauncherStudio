use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Entry, Label, Orientation};

pub fn labeled_entry(label: &str) -> (GtkBox, Entry) {
    let row = GtkBox::new(Orientation::Horizontal, 8);
    let lbl = Label::new(Some(label));
    lbl.set_xalign(0.0);
    let entry = Entry::new();
    entry.set_hexpand(true);
    row.append(&lbl);
    row.append(&entry);
    (row, entry)
}
