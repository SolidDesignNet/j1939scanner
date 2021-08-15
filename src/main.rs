extern crate gio;
extern crate gtk;

use std::rc::Rc;
use std::sync::{Arc, Mutex};

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
use j1939da_ui::J1939Table;
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

fn create_application(bus: MultiQueue<J1939Packet>) -> Result<Application> {
    let application = Application::new(Some("net.soliddesign.j1939dascanner"), Default::default());
    application.connect_activate(move |app| {
        let window = ApplicationWindow::new(app);
        window.set_title("J1939DA Tool - Solid Design");
        window.set_default_size(800, 600);

        let notebook = Notebook::new();

        let j1939_table = Rc::new(Mutex::new(J1939Table::new()));
        let j1939da_table_component = j1939da_ui::create_ui(j1939_table.clone());
        notebook.append_page(
            &j1939da_table_component,
            Some(&gtk::Label::new(Some(&"J1939DA"))),
        );

        notebook.append_page(
            &j1939da_ui::j1939da_log(&bus),
            Some(&gtk::Label::new(Some(&"CAN"))),
        );

        let menubar = MenuBar::new();
        {
            let files_item = MenuItem::with_label("Files");
            let menu = Menu::new();
            menu.append(
                &create_j1939da_menu(&j1939_table, &window).expect("Unable to create J1939 menu"),
            );
            files_item.set_submenu(Some(&menu));
            menubar.append(&files_item);
        }
        {
            let rp1210_menu = MenuItem::with_label("RP1210");
            rp1210_menu.set_submenu(create_rp1210_menu(bus.clone()).ok().as_ref());
            menubar.append(&rp1210_menu);
        }
        let vbox = Box::builder().orientation(Orientation::Vertical).build();
        vbox.pack_start(&menubar, false, false, 0);
        vbox.pack_end(&notebook, true, true, 0);
        window.add(&vbox);
        window.show_all();
    });
    Ok(application)
}

fn create_j1939da_menu(
    j1939_table: &Rc<Mutex<J1939Table>>,
    window: &ApplicationWindow,
) -> Result<MenuItem> {
    let j1939_menu = MenuItem::with_label("J1939DA...");
    let j1939_table = j1939_table.clone();
    j1939_menu.connect_activate(glib::clone!(@weak window => move |_| {
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Open File"),
            Some(&window),
            gtk::FileChooserAction::Open,
        );
        file_chooser.add_buttons(&[
            ("Open", gtk::ResponseType::Ok),
            ("Cancel", gtk::ResponseType::Cancel),
        ]);
        let j1939_table = j1939_table.clone();
        file_chooser.connect_response( move |file_chooser, response| {
            if response == gtk::ResponseType::Ok {
                let filename = file_chooser.filename().expect("Couldn't get filename");
                let filename = filename.to_str();
                filename.map(|f|{
                    j1939_table.lock().expect("Unable to unlock model.")
                    .file(f).expect("Unable to load J1939DA");
                });
                        }
            file_chooser.close();
        });

        file_chooser.show_all();
    }));
    return Ok(j1939_menu);
}

fn create_rp1210_menu(bus: MultiQueue<J1939Packet>) -> Result<Menu> {
    let rp1210_menu = Menu::new();
    let closer: Arc<Mutex<Option<std::boxed::Box<dyn Fn() -> ()>>>> = Arc::new(Mutex::new(None));
    {
        // Add the close RP1210 option
        let device_menu_item = MenuItem::with_label("Disconnect");
        let c1 = closer.clone();
        device_menu_item.connect_activate(move |_| {
            let mut closer = c1.lock().unwrap();
            // execute close if there is a prior rp1210 adapter
            closer.as_ref().map(|a| a());
            *closer = None;
        });
        rp1210_menu.append(&device_menu_item);
    }
    for product in rp1210_parsing::list_all_products()? {
        let product_menu_item = MenuItem::with_label(&product.id);
        rp1210_menu.append(&product_menu_item);
        let product_menu = Menu::new();

        // Add all RP1210 J1939 devices
        for device in product.devices {
            let device_menu_item = MenuItem::with_label(&device.description);
            let pid = product.id.clone();
            let bus = bus.clone();
            let closer = closer.clone();
            device_menu_item.connect_activate(move |_| {
                let mut closer = closer.lock().unwrap();
                // execute close if there is a prior rp1210 adapter
                closer.as_ref().map(|a| a());
                // create a new adapter
                let rp1210 = Rp1210::new(&pid, bus.clone()).unwrap();
                *closer = Some(rp1210.run(device.id, "J1939:Baud=Auto", 0xF9).ok().unwrap());
            });
            product_menu.add(&device_menu_item);
        }
        product_menu_item.set_submenu(Some(&product_menu));
    }

    Ok(rp1210_menu)
}
