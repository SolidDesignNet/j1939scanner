use anyhow::*;
use core::cell::RefCell;
use simple_table::simple_table::{SimpleModel, SimpleTable};
use std::rc::Rc;

use fltk::frame::Frame;
use fltk::group::{Pack, PackType, Scroll};
use fltk::input::Input;
use fltk::prelude::{GroupExt, InputExt, WidgetExt};

use crate::j1939::J1939DARow;
use crate::{Layout, Layoutable};

#[derive(Default)]
pub struct J1939Table {
    rows: Vec<J1939DARow>,
    filtered: Vec<usize>,
    spn_dec: String,
    spn_hex: String,
    pgn_dec: String,
    pgn_hex: String,
    description: Vec<String>,
    update_cb: Option<Box<dyn FnMut()>>,
}

impl J1939Table {
    pub fn file(&mut self, file: &str) -> anyhow::Result<()> {
        self.rows = crate::j1939::load_j1939da(file)?;
        self.refilter();
        Ok(())
    }
    pub fn filtered_row_count(&self) -> usize {
        self.filtered.len()
    }
    pub fn filtered_row(&self, row: usize) -> &J1939DARow {
        &self.rows[self.filtered[row]]
    }
    pub fn refilter(&mut self) {
        let table = self;
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

        println!(
            "refilter spns: {:?} pgns: {:?} desc: {:?}",
            spns, pgns, table.description
        );
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
        println!("filtered.len {}", table.filtered.len());
    }
}
pub fn create_ui(rc_self: Rc<RefCell<J1939Table>>, layout: &mut Layout) {
    rc_self.borrow_mut().refilter();

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
            rc.borrow_mut().pgn_dec = e.value();
            rc.borrow_mut().refilter();
        });

        let mut pgn_hex = Input::default().layout_right(&mut layout_pgn, 80);
        let rc = rc_self.clone();
        pgn_hex.set_callback(move |e| {
            rc.borrow_mut().pgn_hex = e.value();
            rc.borrow_mut().refilter();
        });
        //filter description
        let mut description = Input::default().layout_top(&mut layout_pgn, 80);
        let rc = rc_self.clone();
        description.set_callback(move |e| {
            rc.borrow_mut().description = e
                .value()
                .to_ascii_lowercase()
                .split_ascii_whitespace()
                .map(|s| s.to_string())
                .collect();
            rc.borrow_mut().refilter();
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
            rc.borrow_mut().spn_dec = e.value();
            rc.borrow_mut().refilter();
        });
        let mut spn_hex = Input::default().layout_right(&mut spn_layout, 80);
        let rc = rc_self.clone();
        spn_hex.set_callback(move |e| {
            rc.borrow_mut().spn_hex = e.value();
            rc.borrow_mut().refilter();
        });
    }
    filter_box.end();
    let sw = Scroll::default().layout_in(layout, 0);

    let columns: Vec<J1939Column> = vec![
        J1939Column {
            name: "PGN".to_string(),
            width: 50,
            cell: Box::new(move |row| row.pg.map(|p| format!("{:04X}", p))),
        },
        J1939Column {
            name: "Label".to_string(),
            width: 200,
            cell: Box::new(move |row| row.pg_label.to_owned()),
        },
        J1939Column {
            name: "Acronym".to_string(),
            width: 50,
            cell: Box::new(move |row| row.pg_acronym.to_owned()),
        },
        J1939Column {
            name: "SPN".to_string(),
            width: 50,
            cell: Box::new(move |row| row.spn.map(|p| format!("{:04X}", p))),
        },
        J1939Column {
            name: "PGN".to_string(),
            width: 50,
            cell: Box::new(move |row| row.sp_description.to_owned()),
        },
    ];
    let mut simple_table = SimpleTable::new(Box::new(J1939Model {
        j1939_table: rc_self.clone(),
        columns,
        //table: None,
    }));

    rc_self.borrow_mut().update_cb = Some(Box::new(move || simple_table.redraw()));
    sw.end();
    vbox.end();
}

pub struct J1939Model {
    j1939_table: Rc<RefCell<J1939Table>>,
    columns: Vec<J1939Column>,
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
struct J1939Column {
    name: String,
    width: u32,
    cell: Box<dyn Fn(&J1939DARow) -> Option<String>>,
}
