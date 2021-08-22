// yikes. Comment out the next line, then try to make sense of that error message!
use std::collections::HashMap;
use std::{rc::Rc, sync::Mutex, thread};

use crate::{
    j1939::{packet::J1939Packet, J1939DARow},
    multiqueue::MultiQueue,
};

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
            let mut table = rc.lock().expect("Unable to lock.");
            table.pgn_dec = e.buffer().text();
            table.refilter();
        });

        let pgn_hex = gtk::Entry::builder()
            .width_chars(6)
            .placeholder_text(&"hex")
            .build();
        filter_box.pack_start(&pgn_hex, true, true, 0);
        let rc = this.clone();
        pgn_hex.connect_changed(move |e| {
            let mut table = rc.lock().expect("Unable to lock.");
            table.pgn_hex = e.buffer().text();
            table.refilter();
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
            let mut table = rc.lock().expect("Unable to lock.");
            table.spn_dec = e.buffer().text();
            table.refilter();
        });
        filter_box.pack_start(&spn_dec, true, true, 0);

        let spn_hex = gtk::Entry::builder()
            .width_chars(6)
            .placeholder_text(&"hex")
            .build();
        let rc = this.clone();
        spn_hex.connect_changed(move |e| {
            let mut table = rc.lock().expect("Unable to lock.");
            table.spn_hex = e.buffer().text();
            table.refilter();
        });
        filter_box.pack_start(&spn_hex, true, true, 0);
    }
    {
        //filter description
        filter_box.add(&gtk::Label::new(Some("Filter")));

        let desc = gtk::Entry::builder()
            .width_chars(12)
            .placeholder_text(&"description")
            .build();
        let rc = this.clone();
        desc.connect_changed(move |e| {
            let mut table = rc.lock().expect("Unable to lock.");
            table.description = e
                .buffer()
                .text()
                .to_ascii_lowercase()
                .split_ascii_whitespace()
                .map(|s| s.to_string())
                .collect();
            table.refilter();
        });
        filter_box.pack_start(&desc, true, true, 0);
    }
    connect_selectall_copy(&view);

    let sw = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    sw.add(&view);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&filter_box, false, false, 4);
    vbox.pack_start(&sw, true, true, 0);
    vbox.upcast()
}

pub(crate) fn j1939da_log(bus: &MultiQueue<J1939Packet>) -> gtk::Container {
    let list = ListStore::new(&[
        f64::static_type(),
        u32::static_type(),
        String::static_type(),
        String::static_type(),
    ]);
    let view = TreeView::with_model(&TreeModelSort::new(&list));

    view.append_column(&config_col(&"Time (ms)", false, 0));
    view.append_column(&config_col(&"Size", false, 1));
    view.append_column(&config_col(&"Head", true, 2));
    view.append_column(&config_col(&"Data", true, 3));

    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let stream = bus.iter_for(std::time::Duration::from_secs(60 * 60 * 24 * 30));
    thread::spawn(move || stream.for_each(|packet| tx.send(packet).unwrap()));
    rx.attach(None, move |packet| {
        list.insert_with_values(
            None,
            &[
                (0, &packet.time()),
                (1, &(packet.data().len() as u32)),
                (2, &packet.header()),
                (3, &packet.data_str()),
            ],
        );
        glib::Continue(true)
    });

    connect_selectall_copy(&view);

    let sw = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    sw.add(&view);
    sw.upcast()
}

fn connect_selectall_copy(view: &TreeView) {
    view.selection().set_mode(SelectionMode::Multiple);
    view.connect_key_press_event(|view, key| {
        if key.state().contains(gdk::ModifierType::CONTROL_MASK)
            && key.keyval() == gdk::keys::Key::from_unicode('a')
        {
            view.selection().select_all();
            Inhibit(true)
        } else if key.state().contains(gdk::ModifierType::CONTROL_MASK)
            && key.keyval() == gdk::keys::Key::from_unicode('c')
        {
            copy_table_to_clipboard(&view);
            Inhibit(true)
        } else {
            Inhibit(false)
        }
    });
}

fn copy_table_to_clipboard(view: &TreeView) {
    let (vec, list) = view.selection().selected_rows();
    let as_string = vec
        .iter()
        .map(|path| {
            (0..list.n_columns())
                .map(|column| {
                    let value = list.value(list.iter(path).as_ref().unwrap(), column);
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
        .fold(
            String::new(),
            |a, b| if a.is_empty() { b } else { a + "\n" + &b },
        );
    gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD).set_text(&as_string);
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
                let empty = "".to_string();
                self.list.insert_with_values(
                    None,
                    &[
                        (0, &row.pg.unwrap_or(0)),
                        (1, &format!("{:04X}", row.pg.unwrap_or(0))),
                        (2, &row.spn.unwrap_or(0)),
                        (3, &format!("{:04X}", row.spn.unwrap_or(0))),
                        (4, row.pg_description.as_ref().unwrap_or(&empty)),
                        (5, row.sp_label.as_ref().unwrap_or(&empty)),
                        (6, row.unit.as_ref().unwrap_or(&empty)),
                        (7, &row.scale.unwrap_or(1.0)),
                        (8, &row.offset.unwrap_or(0.0)),
                    ],
                );
            }
        }
    }
}
