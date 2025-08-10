use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Entry, Label, Orientation};

pub fn labeled_entry(label: &str) -> (GtkBox, Entry) {
    let row = GtkBox::new(Orientation::Horizontal, 8);
    // Right-align the label with a consistent width for clean alignment
    let lbl = Label::new(Some(label));
    lbl.set_halign(gtk4::Align::End);
    lbl.set_xalign(1.0);
    lbl.set_width_chars(18);
    let entry = Entry::new();
    entry.set_hexpand(true);
    row.append(&lbl);
    row.append(&entry);
    (row, entry)
}
