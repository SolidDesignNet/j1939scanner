use crate::layout::Layoutable;

use std::sync::{Arc, Mutex};

// yikes. Comment out the next line, then try to make sense of that error message!
use anyhow::*;

mod j1939;
mod j1939da_ui;
mod layout;
mod multiqueue;
#[cfg_attr(not(target_os = "windows"), path = "sim.rs")]
#[cfg_attr(target_os = "windows", path = "rp1210.rs")]
mod rp1210;
mod rp1210_parsing;

use fltk::{
    self,
    app::*,
    button::Button,
    dialog,
    enums::{FrameType, Shortcut},
    group::*,
    menu::{MenuFlag, SysMenuBar},
    prelude::*,
    window::Window,
    *,
};
use j1939::packet::*;
use j1939da_ui::J1939DaData;
use layout::Layout;
use multiqueue::*;
use rp1210::*;

pub fn main() -> Result<()> {
    //create abstract CAN bus
    let bus: MultiQueue<J1939Packet> = MultiQueue::new();

    // log everything
    //bus.log();

    // UI
    run(bus)?;

    Err(anyhow!("Application should not stop running."))
}
fn run(bus: MultiQueue<J1939Packet>) -> Result<(), FltkError> {
    let application = App::default();
    let j1939_data = Arc::new(Mutex::new(J1939DaData::default()));

    let mut layout = Layout::new(800, 600);
    let layout = &mut layout;
    let mut window = Window::default()
        .with_label("J1939DA Tool - Solid Design")
        .layout_in(layout);

    {
        // window content
        {
            // menu
            let mut menu = menu::SysMenuBar::default().layout_top(layout, 20);
            menu.set_frame(FrameType::FlatBox);

            let j1939_table = j1939_data.clone();
            menu.add(
                "Files/J1939DA...",
                Shortcut::None,
                MenuFlag::Normal,
                move |_m| select_j1939da_file(&mut j1939_table.lock().unwrap()).unwrap(),
            );

            menu.add("RP1210", Shortcut::None, MenuFlag::Submenu, |_| {});
            create_rp1210_menu(bus.clone(), &mut menu).unwrap(); // don't unwrap or it will fail on Linux
            menu.end();
        }

        let tabs = Tabs::default().layout_in(layout);
        layout.y += 25;
        {
            let grp = Group::default().with_label("J1939DA\t\t").layout_in(layout);
            j1939da_ui::create_ui(j1939_data.clone(), layout);
            grp.end();
        }
        {
            let grp = Group::default().with_label("CAN\t\t").layout_in(layout);
            let pack = Pack::default().layout_in(layout);
            Button::default()
                .with_label("CAN CAN")
                .layout_top(layout, 20);
            //j1939da_ui::j1939da_log(&bus);
            pack.end();
            grp.end();
        }
        tabs.end();
    }
    window.make_resizable(true);
    window.end();
    window.show();
    application.run()
}

// to be executed when switching adapter options.
type AdapterCloser = Arc<Mutex<Option<std::boxed::Box<dyn Fn()>>>>;

fn create_rp1210_menu(bus: MultiQueue<J1939Packet>, menu: &mut SysMenuBar) -> Result<()> {
    let closer: AdapterCloser = Arc::new(Mutex::new(None));
    // Add the close RP1210 option
    {
        let closer = closer.clone();
        menu.add(
            "RP1210/Disconnect",
            Shortcut::None,
            MenuFlag::Normal,
            move |_| {
                let mut closer = closer.lock().unwrap();
                // execute close if there is a prior rp1210 adapter
                if let Some(adapter) = closer.as_ref() {
                    adapter()
                }
                *closer = None;
            },
        );
    }

    for product in rp1210_parsing::list_all_products()? {
        // Add all RP1210 J1939 devices
        for device in product.devices {
            let pid = product.id.clone();
            let bus = bus.clone();
            let closer = closer.clone();
            menu.add(
                &("RP1210/".to_string() + &product.description + "/" + &device.description),
                Shortcut::None,
                MenuFlag::Normal,
                move |_| {
                    let mut closer = closer.lock().unwrap();
                    // execute close if there is a prior rp1210 adapter
                    if let Some(adapter) = closer.as_ref() {
                        adapter()
                    }
                    // create a new adapter
                    let mut rp1210 = Rp1210::new(&pid, bus.clone()).unwrap();
                    *closer = Some(rp1210.run(device.id, "J1939:Baud=Auto", 0xF9).ok().unwrap());
                },
            );
        }
    }

    Ok(())
}

fn select_j1939da_file(j1939_table: &mut J1939DaData) -> Result<()> {
    let mut chooser = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
    chooser.set_filter("*.xlsx");
    chooser.set_title("Select J1939DA");
    chooser.show();

    if let Some(file) = chooser.filename().to_str() {
        j1939_table.file(file).expect("Unable to load J1939DA");
    }

    Ok(())
}
