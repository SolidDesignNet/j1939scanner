extern crate gio;
extern crate gtk;

// yikes. Comment out the next line, then try to make sense of that error message!
use anyhow::*;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::*;
use std::collections::HashMap;

mod j1939;
mod j1939da_ui;
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
    let bus: MultiQueue<J1939Packet> = MultiQueue::new();

    // log everything
    //bus.log();

    // load RP1210 driver and attach to bus
    //    let mut rp1210 = Rp1210::new("NULN2R32", bus.clone())?;
    let mut rp1210 = Rp1210::new("NXULNK32", bus.clone())?;

    // select first device, J1939 and collect packets
    rp1210.run(1, "J1939:Baud=Auto", 0xF9)?;

    // UI
    create_application(bus.clone()).run();

    Err(anyhow!("Application should not stop running."))
}

fn create_application(bus: MultiQueue<J1939Packet>) -> Application {
    let application =
        Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default());
    application.connect_activate(move |app| {
        let window = ApplicationWindow::new(app);
        window.set_title("Second GTK+ Program");
        window.set_default_size(350, 70);

        let notebook = Notebook::new();
        notebook.append_page(
            &j1939da_ui::j1939da_log(&bus),
            Some(&gtk::Label::new(Some(&"Log"))),
        );
        notebook.append_page(
            &j1939da_ui::j1939da_table(),
            Some(&gtk::Label::new(Some(&"Table"))),
        );
        // notebook.append_page(
        //     &j1939da_ui::j1939da_scanner(&spns, &bus),
        //     Some(&gtk::Label::new(Some(&"Scanner"))),
        // );
        // notebook.append_page(
        //     &j1939da_ui::j1939da_faults(&spns, &bus),
        //     Some(&gtk::Label::new(Some(&"Faults"))),
        // );

        window.add(&notebook);
        window.show_all();
    });
    application
}
