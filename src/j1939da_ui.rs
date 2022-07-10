use simple_table::simple_table::{SimpleModel, SimpleTable};

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use fltk::{
    frame::Frame,
    group::{Pack, PackType, Scroll},
    input::Input,
    prelude::{GroupExt, InputExt, WidgetExt},
};

use crate::{j1939::J1939DARow, Layout, Layoutable};

#[derive(Default)]
pub struct J1939DaData {
    rows: Vec<J1939DARow>,
    filtered: Vec<usize>,
    spn_dec: String,
    spn_hex: String,
    pgn_dec: String,
    pgn_hex: String,
    description: Vec<String>,
    simple_table: Option<Arc<Mutex<SimpleTable>>>,
}

impl J1939DaData {
    // load from XLS file
    pub fn file(&mut self, file: &str) -> anyhow::Result<()> {
        self.rows = crate::j1939::load_j1939da(file)?;
        self.refilter();
        Ok(())
    }
    // how many rows after applying filters
    pub fn filtered_row_count(&self) -> usize {
        self.filtered.len()
    }
    // row by filtered index
    pub fn filtered_row(&self, index: usize) -> &J1939DARow {
        &self.rows[self.filtered[index]]
    }
    // reapply filters
    fn refilter(&mut self) {
        let table: &mut J1939DaData = self;
        let pat = |c: char| !c.is_ascii_hexdigit();

        let spns: Vec<u32> = table
            .spn_dec
            .split(pat)
            .map(|s| s.parse())
            .chain(table.spn_hex.split(pat).map(|s| u32::from_str_radix(s, 16)))
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap())
            .collect();
        let pgns: Vec<u32> = table
            .pgn_dec
            .split(pat)
            .map(|s| s.parse())
            .chain(table.pgn_hex.split(pat).map(|s| u32::from_str_radix(s, 16)))
            .filter(|r| r.is_ok())
            .map(|r| r.unwrap())
            .collect();

        table.filtered.clear();
        for index in 0..table.rows.len() {
            let row = &table.rows[index];
            if spns.is_empty() && pgns.is_empty() && table.description.is_empty()
                || row.spn.filter(|n| spns.contains(n)).is_some()
                || row.pg.filter(|n| pgns.contains(n)).is_some()
                || row
                    .pg_description
                    .as_ref()
                    .filter(|desc| {
                        table
                            .description
                            .iter()
                            .any(|token| desc.to_ascii_lowercase().contains(token))
                    })
                    .is_some()
                || row
                    .sp_description
                    .as_ref()
                    .filter(|desc| {
                        table
                            .description
                            .iter()
                            .any(|token| desc.to_ascii_lowercase().contains(token))
                    })
                    .is_some()
            {
                table.filtered.push(index);
            }
        }
        table.redraw();
    }
    fn pgn_dec(&mut self, v: String) {
        self.pgn_dec = v;
        self.refilter();
    }
    fn pgn_hex(&mut self, v: String) {
        self.pgn_hex = v;
        self.refilter();
    }
    fn spn_dec(&mut self, v: String) {
        self.spn_dec = v;
        self.refilter();
    }
    fn spn_hex(&mut self, v: String) {
        self.spn_hex = v;
        self.refilter();
    }
    fn description(&mut self, v: Vec<String>) {
        self.description = v;
        self.refilter();
    }
    fn redraw(&mut self) {
        let simple_table = self.simple_table.as_ref().unwrap().clone();
        fltk::app::awake_callback(move || simple_table.lock().unwrap().redraw())
    }
}

pub fn create_ui(rc_self: Rc<RefCell<J1939DaData>>, layout: &mut Layout) {
    {
        let vbox = Pack::default().layout_in(layout, 5);
        let filter_box = Pack::default()
            .with_type(PackType::Horizontal)
            .layout_top(layout, 32);
        {
            let mut layout_pgn = *layout;
            // PGN filters
            let label = Frame::default().layout_top(layout, 40).with_label("PGN");
            label.layout_right(&mut layout_pgn, 60);

            let mut pgn_dec = Input::default().layout_right(&mut layout_pgn, 80);
            let rc = rc_self.clone();
            pgn_dec.set_callback(move |e| {
                (*rc).borrow_mut().pgn_dec(e.value());
            });

            let mut pgn_hex = Input::default().layout_right(&mut layout_pgn, 80);
            let rc = rc_self.clone();
            pgn_hex.set_callback(move |e| {
                (*rc).borrow_mut().pgn_hex(e.value());
            });
            //filter description
            let mut description = Input::default().layout_top(&mut layout_pgn, 80);
            let rc = rc_self.clone();
            description.set_callback(move |e| {
                (*rc).borrow_mut().description(
                    e.value()
                        .to_ascii_lowercase()
                        .split_ascii_whitespace()
                        .map(|s| s.to_string())
                        .collect(),
                );
            });
        }
        filter_box.end();
        let filter_box = Pack::default()
            .with_type(PackType::Horizontal)
            .layout_top(layout, 32);
        {
            println!("SPN {:?}", layout);
            // SPN filters
            let label = Frame::default().layout_top(layout, 60).with_label("SPN");
            let mut spn_layout = *layout;
            label.layout_right(&mut spn_layout, 60);

            let mut spn_dec = Input::default().layout_right(&mut spn_layout, 80);
            let rc = rc_self.clone();
            spn_dec.set_callback(move |e| {
                (*rc).borrow_mut().spn_dec(e.value());
            });

            let mut spn_hex = Input::default().layout_right(&mut spn_layout, 80);
            let rc = rc_self.clone();
            spn_hex.set_callback(move |e| {
                (*rc).borrow_mut().spn_hex(e.value());
            });
        }
        filter_box.end();
        let sw = Scroll::default().layout_in(layout, 0);
        (*rc_self).borrow_mut().simple_table = Some(Arc::new(Mutex::new(SimpleTable::new(
            Box::new(J1939Model {
                j1939_table: rc_self.clone(),
                columns: vec![
                    J1939DaColumn {
                        name: "PGN".to_string(),
                        width: 50,
                        cell: Box::new(move |row| row.pg.map(|p| format!("{:04X}", p))),
                    },
                    J1939DaColumn {
                        name: "Label".to_string(),
                        width: 200,
                        cell: Box::new(move |row| row.pg_label.to_owned()),
                    },
                    J1939DaColumn {
                        name: "Acronym".to_string(),
                        width: 50,
                        cell: Box::new(move |row| row.pg_acronym.to_owned()),
                    },
                    J1939DaColumn {
                        name: "SPN".to_string(),
                        width: 50,
                        cell: Box::new(move |row| row.spn.map(|p| format!("{:04X}", p))),
                    },
                    J1939DaColumn {
                        name: "PGN".to_string(),
                        width: 50,
                        cell: Box::new(move |row| row.sp_description.to_owned()),
                    },
                ],
            }),
        ))));

        sw.end();
        vbox.end();
    }
    (*rc_self).borrow_mut().refilter();
}

// simple_table J1939DaData model
pub struct J1939Model {
    j1939_table: Rc<RefCell<J1939DaData>>,
    columns: Vec<J1939DaColumn>,
}

impl SimpleModel for J1939Model {
    fn row_count(&mut self) -> usize {
        self.j1939_table.borrow().filtered_row_count()
    }

    fn column_count(self: &mut J1939Model) -> usize {
        self.columns.len()
    }

    fn header(self: &mut J1939Model, col: usize) -> String {
        self.columns[col].name.clone()
    }

    fn column_width(self: &mut J1939Model, col: usize) -> u32 {
        self.columns[col].width
    }

    fn cell(self: &mut J1939Model, row: i32, col: i32) -> Option<String> {
        (self.columns[col as usize].cell)(self.j1939_table.borrow().filtered_row(row as usize))
    }
}
struct J1939DaColumn {
    name: String,
    width: u32,
    cell: Box<dyn Fn(&J1939DARow) -> Option<String> + Send>,
}
