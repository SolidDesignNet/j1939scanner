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

pub fn j1939da_table(table: &HashMap<u16, J1939DARow>) -> gtk::Container {
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

    for row in table.values() {
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

    let sw = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    sw.add(&view);
    sw.upcast()
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
