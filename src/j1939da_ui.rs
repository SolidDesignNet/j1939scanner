extern crate gio;
extern crate glib;
extern crate gtk;

// yikes. Comment out the next line, then try to make sense of that error message!
use gio::prelude::*;
use gtk::prelude::*;
use gtk::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
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
    let controller = Rc::new(Controller {
        table: crate::j1939::load_j1939da("da.xlsx").unwrap(),
        list: ListStore::new(&[
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
        ]),
        spn_dec: "".to_string(),
        spn_hex: "".to_string(),
        pgn_dec: "".to_string(),
        pgn_hex: "".to_string(),
    });

    let view = TreeView::with_model(&controller.list);
    view.append_column(&config_col(&"PGN", 0));
    view.append_column(&config_col(&"SPN", 1));
    view.append_column(&config_col(&"Name", 2));
    view.append_column(&config_col(&"Value", 3));
    view.append_column(&config_col(&"Unit", 4));

    controller.refilter();

    let filter_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    {
        // SPN filters
        filter_box.add(&gtk::Label::new(Some("SPN filter")));

        let spn_dec = gtk::Entry::builder()
            .width_chars(6)
            .placeholder_text(&"decimal")
            .build();
        let c2 = controller.clone();
        spn_dec.connect_changed(move |e| {
            c2.spn_dec = e.buffer().text();
            c2.refilter();
        });
        filter_box.pack_start(&spn_dec, true, true, 0);

        let spn_hex = gtk::Entry::builder()
            .width_chars(6)
            .placeholder_text(&"hex")
            .build();
        let c2 = controller.clone();
        spn_hex.connect_changed(move |e| {
            c2.spn_hex = e.buffer().text();
            c2.refilter();
        });
        filter_box.pack_start(&spn_hex, true, true, 0);
    }
    {
        // PGN filters
        filter_box.pack_start(&gtk::Label::new(Some("PGN filter")), false, true, 0);
        let pgn_dec = gtk::Entry::builder()
            .width_chars(6)
            .placeholder_text(&"decimal")
            .build();
        filter_box.pack_start(&pgn_dec, true, true, 0);
        let c2 = controller.clone();
        pgn_dec.connect_changed(move |e| {
            c2.pgn_dec = e.buffer().text();
            c2.refilter();
        });

        let pgn_hex = gtk::Entry::builder()
            .width_chars(6)
            .placeholder_text(&"hex")
            .build();
        filter_box.pack_start(&pgn_hex, true, true, 0);
        let c2 = controller.clone();
        pgn_hex.connect_changed(move |e| {
            c2.pgn_hex = e.buffer().text();
            c2.refilter();
        });
    }
    let sw = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    sw.add(&view);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&filter_box, false, false, 4);
    vbox.pack_start(&sw, true, true, 0);

    vbox.upcast()
}
struct Controller {
    table: HashMap<u16, J1939DARow>,
    list: ListStore,
    spn_dec: String,
    spn_hex: String,
    pgn_dec: String,
    pgn_hex: String,
}
impl Controller {
    fn tokenize(str: &String) -> Vec<String> {
        vec![]
    }
    fn refilter(&self) {
        println!("refilter");
        let spns = tokenize(&self.spn_dec)
            .map(|s| s.parse().ok())
            .append(tokenize(self.spn_hex).map(|s| u16::from_str_radix(s.as_str(), 16).ok()));
        self.list.clear();
        for row in self.table.values() {
            if todo!() {
                self.list.insert_with_values(
                    None,
                    &[
                        (0, &row.spn.unwrap().to_string()),
                        (1, &format!("{:x}", row.spn.unwrap())),
                        (2, &row.sp_label),
                        (3, &row.unit),
                        (4, &""),
                    ],
                );
            }
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
