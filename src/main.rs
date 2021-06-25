extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::*;
use std::error::Error;

mod j1939data;

//impl TreeModelExt for SignalModel {}

fn config_col(name: &str, id: i32) -> TreeViewColumn {
    let number_col = TreeViewColumnBuilder::new().title(name).build();
    let cell = CellRendererText::new();
    //number_col.set_cell_func(cell, rand_cell);
    number_col.pack_start(&cell, true);
    number_col.add_attribute(&cell, "text", id);
    number_col
}

pub fn main() {
    j1939data::load_j1939da("da.xlsx".to_string());

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
        // for (_spn, row) in &table {
        //     list.insert_with_values(
        //         None,
        //         &[0, 1, 2, 3, 4],
        //         &[&row.pgn_label(), &row.spn_label(), &row.name, &"", &""],
        //     );
        // }

        let view = TreeView::with_model(&list);

        view.append_column(&config_col(&"PGN", 0));
        view.append_column(&config_col(&"SPN", 1));
        view.append_column(&config_col(&"Name", 2));
        view.append_column(&config_col(&"Value", 3));
        view.append_column(&config_col(&"Unit", 4));

        // let sw = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
        // sw.add(&view);
        window.add(&view);

        window.show_all();
    });

    application.run(&[]);
}
