extern crate gio;
extern crate gtk;

// yikes. Comment out the next line, then try to make sense of that error message!
use anyhow::*;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::*;

mod j1939;
mod j1939da_ui;
mod multiqueue;
#[cfg_attr(not(target_os = "windows"), path = "sim.rs")]
#[cfg_attr(target_os = "windows", path = "rp1210.rs")]
mod rp1210;
mod rp1210_parsing;

use j1939::packet::*;
use multiqueue::*;
use rp1210::*;

pub fn main() -> Result<()> {
    //create abstract CAN bus
    let bus: MultiQueue<J1939Packet> = MultiQueue::new();

    // log everything
    //bus.log();

    // UI
    create_application(bus.clone())?.run();

    Err(anyhow!("Application should not stop running."))
}

fn load_adapter(prod: &str, id: i16, bus: MultiQueue<J1939Packet>) -> Result<i16> {
    // load RP1210 driver and attach to bus
    //    let mut rp1210 = Rp1210::new("NULN2R32", bus.clone())?;
    let mut rp1210 = Rp1210::new(prod, bus)?;

    // select first device, J1939 and collect packets
    rp1210.run(id, "J1939:Baud=Auto", 0xF9)
}
fn create_application(bus: MultiQueue<J1939Packet>) -> Result<Application> {
    let application =
        Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default());
    application.connect_activate(move |app| {
        let window = ApplicationWindow::new(app);
        window.set_title("Second GTK+ Program");
        window.set_default_size(800, 600);

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

        let menu = MenuItem::with_label("RP1210");
        menu.set_submenu(Some(&create_rp1210_menu(bus.clone()).unwrap()));

        let menubar = MenuBar::new();
        menubar.append(&menu);

        let vbox = Box::builder().orientation(Orientation::Vertical).build();
        vbox.pack_start(&menubar, false, false, 0);
        vbox.pack_end(&notebook, true, true, 0);
        window.add(&vbox);
        window.show_all();
    });
    Ok(application)
}

fn create_rp1210_menu(bus: MultiQueue<J1939Packet>) -> Result<Menu> {
    let rp1210_menu = Menu::new();

    for product in rp1210_parsing::list_all_products()? {
        let product_menu_item = MenuItem::with_label(&product.id);
        rp1210_menu.append(&product_menu_item);
        let product_menu = Menu::new();
        for device in product.devices {
            let device_menu_item = MenuItem::with_label(&device.description);
            let pid = product.id.clone();
            let bus = bus.clone();
            device_menu_item.connect_activate(move |_| {
                load_adapter(&pid, device.id, bus.clone()).unwrap();
            });
            product_menu.add(&device_menu_item);
        }
        product_menu_item.set_submenu(Some(&product_menu));
    }

    Ok(rp1210_menu)
}
