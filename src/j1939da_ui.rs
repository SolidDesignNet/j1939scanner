extern crate gio;
extern crate glib;
extern crate gtk;

// yikes. Comment out the next line, then try to make sense of that error message!
use gio::prelude::*;
use gtk::prelude::*;
use gtk::*;
use std::collections::HashMap;
use std::thread;

use crate::j1939::packet::J1939Packet;
use crate::j1939::J1939DARow;
use crate::multiqueue::MultiQueue;

fn config_col(name: &str, id: i32) -> TreeViewColumn {
    let number_col = TreeViewColumnBuilder::new().title(name).build();
    let cell = CellRendererText::new();
    //number_col.set_cell_func(cell, rand_cell);
    number_col.pack_start(&cell, true);
    number_col.add_attribute(&cell, "text", id);
    number_col
}

pub(crate) fn j1939da_log(bus: &MultiQueue<J1939Packet>) -> gtk::Container {
    let list = ListStore::new(&[
        f64::static_type(),
        u32::static_type(),
        String::static_type(),
        String::static_type(),
    ]);
    let view = TreeView::with_model(&list);

    view.append_column(&config_col(&"Time", 0));
    view.append_column(&config_col(&"Size", 1));
    view.append_column(&config_col(&"Head", 2));
    view.append_column(&config_col(&"Data", 3));

    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let stream = bus.iter_for(std::time::Duration::from_secs(60 * 60 * 24 * 30));
    thread::spawn(move || stream.for_each(|p| tx.send(p).unwrap()));
    rx.attach(None, move |p| {
        list.insert_with_values(
            None,
            &[
                (0, &p.time()),
                (1, &(p.data().len() as u32)),
                (2, &p.header()),
                (3, &p.data_str()),
            ],
        );
        glib::Continue(true)
    });

    let sw = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    sw.add(&view);
    sw.upcast()
}

pub fn j1939da_table() -> gtk::Container {
    let list = ListStore::new(&[
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);

    let view = TreeView::with_model(&list);
    view.append_column(&config_col(&"PGN", 0));
    view.append_column(&config_col(&"SPN", 1));
    view.append_column(&config_col(&"Name", 2));
    view.append_column(&config_col(&"Value", 3));
    view.append_column(&config_col(&"Unit", 4));

    let table = crate::j1939::load_j1939da("da.xlsx").unwrap();
    refilter(&table, &list, &|row| true);

    let filter_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    filter_box.add(&gtk::Label::new(Some("SPN filter")));
    filter_box.pack_start(
        &gtk::Entry::builder()
            .width_chars(6)
            .placeholder_text(&"decimal")
            .build(),
        true,
        true,
        0,
    );
    let spn_hex = gtk::Entry::builder()
        .width_chars(6)
        .placeholder_text(&"hex")
        .build();
    let b = spn_hex.buffer();
    spn_hex.connect_changed(move |_| {
        let v = 123; // parse hex FIXME
        refilter(&table, &list, &|row: &J1939DARow| {
            row.spn.map_or_else(|| false, |s| s == v)
        });
    });
    filter_box.pack_start(&spn_hex, true, true, 0);

    filter_box.pack_start(&gtk::Label::new(Some("PGN filter")), false, true, 0);
    filter_box.pack_start(
        &gtk::Entry::builder()
            .width_chars(6)
            .placeholder_text(&"decimal")
            .build(),
        true,
        true,
        0,
    );
    let pgn_hex = gtk::Entry::builder()
        .width_chars(6)
        .placeholder_text(&"hex")
        .build();
    filter_box.pack_start(&pgn_hex, true, true, 0);

    let sw = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    sw.add(&view);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&filter_box, false, false, 4);
    vbox.pack_start(&sw, true, true, 0);

    vbox.upcast()
}
fn refilter(table: &HashMap<u16, J1939DARow>, list: &ListStore, f: &dyn Fn(&J1939DARow) -> bool) {
    list.clear();
    for row in table.values() {
        println!("refilter");
        if f(row) {
            list.insert_with_values(
                None,
                &[
                    (0, &row.pg_label),
                    (1, &row.sp_label),
                    (2, &row.unit),
                    (3, &""),
                    (4, &""),
                ],
            );
        }
    }
}
pub(crate) fn j1939da_scanner(
    table: &HashMap<u16, J1939DARow>,
    bus: &MultiQueue<J1939Packet>,
) -> gtk::Container {
    todo!()
}

pub(crate) fn j1939da_faults(
    table: &HashMap<u16, J1939DARow>,
    bus: &MultiQueue<J1939Packet>,
) -> gtk::Container {
    todo!()
}
