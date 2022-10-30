use simple_table::simple_table::{SimpleModel, SimpleTable};

use std::sync::{Arc, Mutex};

use fltk::{
    frame::Frame,
    group::{Pack, PackType},
    input::Input,
    prelude::{GroupExt, InputExt, WidgetExt},
};

use crate::{j1939::J1939DARow, Layout, Layoutable};

#[derive(Default)]
pub struct J1939DaData {
    rows: Vec<J1939DARow>,
    filtered: Vec<usize>,
    spn_dec: Vec<u32>,
    spn_hex: Vec<u32>,
    pgn_dec: Vec<u32>,
    pgn_hex: Vec<u32>,
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

        let spns: Vec<u32> = table
            .spn_dec
            .iter()
            .chain(table.spn_hex.iter())
            .map(|s| *s)
            .collect();
        let pgns: Vec<u32> = table
            .pgn_dec
            .iter()
            .chain(table.pgn_hex.iter())
            .map(|s| *s)
            .collect();

        println!("spns:{:?}", spns);
        println!("pgns:{:?}", pgns);

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
    fn description(&mut self, v: Vec<String>) {
        self.description = v;
        self.refilter();
    }
    fn redraw(&mut self) {
        let simple_table = self.simple_table.as_ref().unwrap().clone();
        fltk::app::awake_callback(move || simple_table.lock().unwrap().redraw())
    }
}

fn parse_dec(v: String) -> Vec<u32> {
    let pat = |c: char| !c.is_ascii_hexdigit();
    v.split(pat)
        .map(|s| s.parse())
        .filter(|s| s.is_ok())
        .map(|s| s.unwrap())
        .collect()
}
fn parse_hex(v: String) -> Vec<u32> {
    let pat = |c: char| !c.is_ascii_hexdigit();
    v.split(pat)
        .map(|s| u32::from_str_radix(s, 16))
        .filter(|s| s.is_ok())
        .map(|s| s.unwrap())
        .collect()
}

pub fn create_ui(rc_self: Arc<Mutex<J1939DaData>>, layout: &mut Layout) {
    let mut vbox = Pack::default()
        .with_type(PackType::Vertical)
        .layout_in(layout);

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
            let ref mut this = (*rc).lock().unwrap();
            this.pgn_dec = parse_dec(e.value());
            if this.pgn_dec.is_empty() {
                e.set_value("decimal");
            }
            this.refilter();
        });

        let mut pgn_hex = Input::default().layout_right(&mut layout_pgn, 80);
        let rc = rc_self.clone();
        pgn_hex.set_callback(move |e| {
            let ref mut this = (*rc).lock().unwrap();
            this.pgn_hex = parse_hex(e.value());
            if this.pgn_dec.is_empty() {
                e.set_value("hex");
            }
            this.refilter();
        });
        //filter description
        let mut description = Input::default().layout_top(&mut layout_pgn, 80);
        let rc = rc_self.clone();
        description.set_callback(move |e| {
            (*rc).lock().unwrap().description(
                e.value()
                    .to_ascii_lowercase()
                    .split_ascii_whitespace()
                    .map(|s| s.to_string())
                    .collect(),
            );
        });
        pgn_dec.set_value("decimal");
        pgn_hex.set_value("hex");
    }
    filter_box.end();
    let filter_box = Pack::default()
        .with_type(PackType::Horizontal)
        .layout_top(layout, 32);
    {
        // SPN filters
        let label = Frame::default().layout_top(layout, 60).with_label("SPN");
        let mut spn_layout = *layout;
        label.layout_right(&mut spn_layout, 60);

        let mut spn_dec = Input::default().layout_right(&mut spn_layout, 80);
        let rc = rc_self.clone();
        spn_dec.set_callback(move |e| {
            let ref mut this = (*rc).lock().unwrap();
            this.spn_dec = parse_dec(e.value());
            if this.spn_dec.is_empty() {
                e.set_value("decimal");
            }
            this.refilter();
        });

        let mut spn_hex = Input::default().layout_right(&mut spn_layout, 80);
        let rc = rc_self.clone();
        spn_hex.set_callback(move |e| {
            let ref mut this = (*rc).lock().unwrap();
            this.spn_hex = parse_hex(e.value());
            if this.spn_dec.is_empty() {
                e.set_value("hex");
            }
            this.refilter();
        });
        spn_dec.set_value("decimal");
        spn_hex.set_value("hex");
    }
    filter_box.end();

    let simple_table = SimpleTable::new(Box::new(J1939Model {
        j1939da_data: rc_self.clone(),
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
                name: "Description".to_string(),
                width: 200,
                cell: Box::new(move |row| row.sp_description.to_owned()),
            },
            J1939DaColumn {
                name: "Unit".to_string(),
                width: 50,
                cell: Box::new(move |row| row.unit.to_owned()),
            },
        ],
    }));
    vbox.add_resizable(&simple_table.table);
    (*rc_self).lock().unwrap().simple_table = Some(Arc::new(Mutex::new(simple_table)));
    vbox.end();

    (*rc_self).lock().unwrap().refilter();
}

// simple_table J1939DaData model
pub struct J1939Model {
    j1939da_data: Arc<Mutex<J1939DaData>>,
    columns: Vec<J1939DaColumn>,
}

impl SimpleModel for J1939Model {
    fn row_count(&mut self) -> usize {
        self.j1939da_data.lock().unwrap().filtered_row_count()
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
        (self.columns[col as usize].cell)(
            self.j1939da_data.lock().unwrap().filtered_row(row as usize),
        )
    }
}
struct J1939DaColumn {
    name: String,
    width: u32,
    cell: Box<dyn Fn(&J1939DARow) -> Option<String> + Send>,
}
