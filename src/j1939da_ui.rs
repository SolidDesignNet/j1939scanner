extern crate gio;
extern crate glib;
extern crate gtk;

// yikes. Comment out the next line, then try to make sense of that error message!
use gio::prelude::*;
use gtk::prelude::*;
use gtk::*;
use std::collections::HashMap;
use std::{rc::Rc, sync::Mutex, thread};

use crate::{
    j1939::{packet::J1939Packet, J1939DARow},
    multiqueue::MultiQueue,
};

fn config_col(name: &str, mono: bool, id: i32) -> TreeViewColumn {
    let col = TreeViewColumn::new();
    col.set_title(name);
    let cell = CellRendererText::new();
    if mono {
        cell.set_font(Some("monospace"));
    }
    cell.set_ellipsize(pango::EllipsizeMode::End);

    col.pack_start(&cell, true);
    col.add_attribute(&cell, "text", id);
    col.set_sort_indicator(true);
    col.set_clickable(true);
    col.set_sort_column_id(id);
    col.set_reorderable(true);
    col.set_resizable(true);

    col
}
pub fn create_ui(this: Rc<Mutex<J1939Table>>) -> gtk::Container {
    let table = this.lock().expect("Unable to lock.");
    let view = TreeView::with_model(&table.list);
    view.append_column(&config_col(&"PGN", false, 0));
    view.append_column(&config_col(&"xPGN", true, 1));
    view.append_column(&config_col(&"SPN", false, 2));
    view.append_column(&config_col(&"xSPN", true, 3));
    view.append_column(&config_col(&"PGN Description", false, 4));
    view.append_column(&config_col(&"SPN Description", false, 5));
    view.append_column(&config_col(&"Unit", false, 6));
    view.append_column(&config_col(&"Scale", false, 7));
    view.append_column(&config_col(&"Offest", false, 8));

    table.refilter();

    let filter_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    {
        // PGN filters
        filter_box.pack_start(&gtk::Label::new(Some("PGN filter")), false, true, 0);
        let pgn_dec = gtk::Entry::builder()
            .width_chars(6)
            .placeholder_text(&"decimal")
            .build();
        filter_box.pack_start(&pgn_dec, true, true, 0);
        let rc = this.clone();
        pgn_dec.connect_changed(move |e| {
            let mut c = rc.lock().expect("Unable to lock.");
            c.pgn_dec = e.buffer().text();
            c.refilter();
        });

        let pgn_hex = gtk::Entry::builder()
            .width_chars(6)
            .placeholder_text(&"hex")
            .build();
        filter_box.pack_start(&pgn_hex, true, true, 0);
        let rc = this.clone();
        pgn_hex.connect_changed(move |e| {
            let mut c = rc.lock().expect("Unable to lock.");
            c.pgn_hex = e.buffer().text();
            c.refilter();
        });
    }
    {
        // SPN filters
        filter_box.add(&gtk::Label::new(Some("SPN filter")));

        let spn_dec = gtk::Entry::builder()
            .width_chars(6)
            .placeholder_text(&"decimal")
            .build();
        let rc = this.clone();
        spn_dec.connect_changed(move |e| {
            let mut c = rc.lock().expect("Unable to lock.");
            c.spn_dec = e.buffer().text();
            c.refilter();
        });
        filter_box.pack_start(&spn_dec, true, true, 0);

        let spn_hex = gtk::Entry::builder()
            .width_chars(6)
            .placeholder_text(&"hex")
            .build();
        let rc = this.clone();
        spn_hex.connect_changed(move |e| {
            let mut c = rc.lock().expect("Unable to lock.");
            c.spn_hex = e.buffer().text();
            c.refilter();
        });
        filter_box.pack_start(&spn_hex, true, true, 0);
    }
    {
        //filter description}
        filter_box.add(&gtk::Label::new(Some("Filter")));

        let desc = gtk::Entry::builder()
            .width_chars(12)
            .placeholder_text(&"description")
            .build();
        let rc = this.clone();
        desc.connect_changed(move |e| {
            let mut c = rc.lock().expect("Unable to lock.");
            c.description = e
                .buffer()
                .text()
                .to_ascii_lowercase()
                .split_ascii_whitespace()
                .map(|s| s.to_string())
                .collect();
            c.refilter();
        });
        filter_box.pack_start(&desc, true, true, 0);
    }
    view.selection().set_mode(SelectionMode::Multiple);

    let sw = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    sw.add(&view);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&filter_box, false, false, 4);
    vbox.pack_start(&sw, true, true, 0);

    add_copy_button(&vbox.upcast(), view)
}

pub(crate) fn j1939da_log(bus: &MultiQueue<J1939Packet>) -> gtk::Container {
    let list = ListStore::new(&[
        f64::static_type(),
        u32::static_type(),
        String::static_type(),
        String::static_type(),
    ]);
    let view = TreeView::with_model(&TreeModelSort::new(&list));

    view.append_column(&config_col(&"Time", false, 0));
    view.append_column(&config_col(&"Size", false, 1));
    view.append_column(&config_col(&"Head", true, 2));
    view.append_column(&config_col(&"Data", true, 3));

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

    view.selection().set_mode(SelectionMode::Multiple);

    let sw = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    sw.add(&view);

    add_copy_button(&sw.upcast(), view)
}

fn add_copy_button(sw: &Container, view: TreeView) -> Container {
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(sw, true, true, 0);
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let copy_button = Button::new();
    copy_button.set_label("Copy");
    copy_button.connect_clicked(move |_f| {
        gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD).set_text(&copy(&view));
    });
    buttons.pack_end(&copy_button, false, false, 0);
    vbox.pack_end(&buttons, false, false, 0);
    vbox.upcast()
}

fn copy(view: &TreeView) -> String {
    let (vec, list) = view.selection().selected_rows();
    vec.iter()
        .map(|p| {
            (0..list.n_columns())
                .map(|c| {
                    let value = list.value(list.iter(p).as_ref().unwrap(), c);
                    value
                        .get::<String>()
                        .map(|s| "\"".to_string() + &s + "\"")
                        .or_else(|_| value.get::<f64>().map(|n| n.to_string()))
                        .or_else(|_| value.get::<f32>().map(|n| n.to_string()))
                        .or_else(|_| value.get::<i32>().map(|n| n.to_string()))
                        .or_else(|_| value.get::<u32>().map(|n| n.to_string()))
                        .unwrap_or("Unknown".to_string())
                })
                .fold(
                    String::new(),
                    |a, b| if a.is_empty() { b } else { a + "\t" + &b },
                )
        })
        .fold(String::new(), |a, b| a + "\n" + &b)
}

pub struct J1939Table {
    table: HashMap<u32, J1939DARow>,
    list: ListStore,
    spn_dec: String,
    spn_hex: String,
    pgn_dec: String,
    pgn_hex: String,
    description: Vec<String>,
}

impl J1939Table {
    pub fn file(&mut self, file: &str) -> anyhow::Result<()> {
        self.table = crate::j1939::load_j1939da(file)?;
        self.refilter();
        Ok(())
    }
    pub fn new() -> J1939Table {
        J1939Table {
            table: HashMap::new(),
            list: ListStore::new(&[
                u32::static_type(),
                String::static_type(),
                u32::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
                f64::static_type(),
                f64::static_type(),
            ]),
            spn_dec: "".to_string(),
            spn_hex: "".to_string(),
            pgn_dec: "".to_string(),
            pgn_hex: "".to_string(),
            description: vec![],
        }
    }

    pub fn refilter(&self) {
        let pat = |c: char| !c.is_ascii_hexdigit();

        let spns: Vec<u32> = self
            .spn_dec
            .split(pat)
            .map(|s| s.parse())
            .chain(self.spn_hex.split(pat).map(|s| u32::from_str_radix(s, 16)))
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap())
            .collect();
        let pgns: Vec<u32> = self
            .pgn_dec
            .split(pat)
            .map(|s| s.parse())
            .chain(self.pgn_hex.split(pat).map(|s| u32::from_str_radix(s, 16)))
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap())
            .collect();

        println!(
            "refilter spns: {:?} pgns: {:?} desc: {:?}",
            spns, pgns, self.description
        );
        self.list.clear();
        for row in self.table.values() {
            if spns.is_empty() && pgns.is_empty() && self.description.is_empty()
                || row.spn.filter(|n| spns.contains(&n)).is_some()
                || row.pg.filter(|n| pgns.contains(&n)).is_some()
                || row
                    .pg_description
                    .as_ref()
                    .filter(|desc| {
                        self.description
                            .iter()
                            .any(|token| desc.to_ascii_lowercase().contains(token))
                    })
                    .is_some()
                || row
                    .sp_description
                    .as_ref()
                    .filter(|desc| {
                        self.description
                            .iter()
                            .any(|token| desc.to_ascii_lowercase().contains(token))
                    })
                    .is_some()
            {
                self.list.insert_with_values(
                    None,
                    &[
                        (0, &row.pg.unwrap_or(0)),
                        (1, &format!("{:04X}", row.pg.unwrap_or(0))),
                        (2, &row.spn.unwrap_or(0)),
                        (3, &format!("{:04X}", row.spn.unwrap_or(0))),
                        (4, &row.pg_description),
                        (5, &row.sp_label),
                        (6, &row.unit),
                        (7, &row.scale.unwrap_or(1.0)),
                        (8, &row.offset.unwrap_or(0.0)),
                    ],
                );
            }
        }
    }
}
