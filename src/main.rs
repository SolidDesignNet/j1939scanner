extern crate gio;
extern crate gtk;

// yikes. Comment out the next line, then try to make sense of that error message!
use gio::prelude::*;

use anyhow::Result;
use gtk::prelude::*;
use gtk::*;

mod j1939;
mod multiqueue;
mod rp1210;

use j1939::packet::Packet;
use j1939::*;
use multiqueue::*;
use rp1210::*;

fn config_col(name: &str, id: i32) -> TreeViewColumn {
    let number_col = TreeViewColumnBuilder::new().title(name).build();
    let cell = CellRendererText::new();
    //number_col.set_cell_func(cell, rand_cell);
    number_col.pack_start(&cell, true);
    number_col.add_attribute(&cell, "text", id);
    number_col
}

pub fn main() -> Result<()> {
    // 10 s buffer of 2,000 packets/s
    let queue: MultiQueue<Packet> = MultiQueue::new(2000 * 10);
    let rp1210 = Rp1210::new("NUL2NXR32".to_string(), queue);

    let table = load_j1939da("da.xlsx".to_string())?;

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
        for (_spn, row) in &table {
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

    application.run(&[]);
    Ok(())
}
