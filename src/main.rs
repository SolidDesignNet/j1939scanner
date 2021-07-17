extern crate gio;
extern crate gtk;

// yikes. Comment out the next line, then try to make sense of that error message!
use anyhow::*;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::*;
use std::collections::HashMap;

mod j1939;
mod multiqueue;
#[cfg_attr(not(target_os = "windows"), path = "sim.rs")]
#[cfg_attr(target_os = "windows", path = "rp1210.rs")]
mod rp1210;

use j1939::packet::*;
use j1939::*;
use multiqueue::*;
use rp1210::*;

pub fn main() -> Result<()> {
    //create abstract CAN bus
    let mut bus: MultiQueue<J1939Packet> = MultiQueue::new();

    // log everything
    bus.log();

    // load RP1210 driver and attach to bus
    //    let mut rp1210 = Rp1210::new("NULN2R32", bus.clone())?;
    let mut rp1210 = Rp1210::new("NXULNK32", &bus)?;

    // select first device, J1939 and collect packets
    rp1210.run(1, "J1939:Baud=Auto", 0xF9)?;

    // test send
    for i in [1..120] {
        std::thread::sleep(std::time::Duration::from_secs(1));
        bus.push(J1939Packet::new(0x18DA00F9, &[0x10, 0x01]));
    }

    // load J1939DA
    let table = load_j1939da("da.xlsx")?;

    // UI
    create_application(table).run(&[]);

    Err(anyhow!("Application should not stop running."))
}

fn config_col(name: &str, id: i32) -> TreeViewColumn {
    let number_col = TreeViewColumnBuilder::new().title(name).build();
    let cell = CellRendererText::new();
    //number_col.set_cell_func(cell, rand_cell);
    number_col.pack_start(&cell, true);
    number_col.add_attribute(&cell, "text", id);
    number_col
}

fn create_application(table: HashMap<u16, J1939DARow>) -> Application {
    let application =
        Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default())
            .expect("failed to initialize GTK application");

    application.connect_activate(move |app| {
        let window = ApplicationWindow::new(app);
        window.set_title("Second GTK+ Program");
        window.set_default_size(350, 70);

        let list = ListStore::new(&[
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
        ]);
        for row in table.values() {
            list.insert_with_values(
                None,
                &[0, 1, 2, 3, 4],
                &[&row.pg_label, &row.sp_label, &row.unit, &"", &""],
            );
        }

        let view = TreeView::with_model(&list);

        view.append_column(&config_col(&"PGN", 0));
        view.append_column(&config_col(&"SPN", 1));
        view.append_column(&config_col(&"Name", 2));
        view.append_column(&config_col(&"Value", 3));
        view.append_column(&config_col(&"Unit", 4));

        let sw = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
        sw.add(&view);
        window.add(&sw);

        window.show_all();
    });
    application
}
