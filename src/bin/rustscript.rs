//! Copyright © Marko Vujnovic, GNU Affero General Public License v3

use rustscript::*;

fn display_the_tui() {
    use cursive::traits::{Nameable, Boxable, Scrollable};
    use cursive::{ align::Align, };

    let mut app = cursive::default();
    let mut panes = cursive::views::LinearLayout::horizontal();
    let mut program_descr = cursive::views::LinearLayout::vertical();
    let picker = main_menu();
    panes.add_child(cursive::traits::Boxable::fixed_size(picker, (30, 25)));
    panes.add_child(cursive::views::DummyView);
    program_descr.add_child(cursive::views::TextView::new(r#"Welcome to Rustscript!"#).align(Align::top_center()));
    program_descr.add_child(cursive::views::DummyView);
    program_descr.add_child(cursive::views::TextView::new(r#"You can see the actions you can perform in the menu on the left. When a menu item is selected a more detailed description of what it does is displayed at the bottom. 
"#)
        .align(Align::top_left())
        .with_name("contents")
        .fixed_size((50, 25)));
    panes.add_child(program_descr);
    app.add_layer(main_layout_wrap(panes));
    app.run();
}

fn main() -> core::result::Result<(), std::io::Error> { tokio::runtime::Runtime::new().unwrap().block_on(async {
    if std::env::args().len() == 1 { display_the_tui(); return Ok(()); }
    let (options, positional_args) = parse_args(std::env::args());
    let script_path = &positional_args[1];
    main_(&script_path).await?;

    Ok(())
})}

fn file_picker<D>(directory: D) -> cursive::views::SelectView<std::fs::DirEntry> where D: AsRef<std::path::Path> {
    let mut view = cursive::views::SelectView::new();
    for entry in std::fs::read_dir(directory).expect("Can't read the directory") {
        if let Ok(e) = entry {
            let file_name = e.file_name().into_string().unwrap();
            view.add_item(file_name, e);
        }
    }
    view.on_select(|app: &mut cursive::Cursive, entry: &std::fs::DirEntry| {
        let mut status_bar = app.find_name::<cursive::views::TextView>("status").unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        let file_size = entry.metadata().unwrap().len();
        let content = format!("{}: {} bytes", file_name, file_size);
        status_bar.set_content(content);
    }).on_submit(|app: &mut cursive::Cursive, entry: &std::fs::DirEntry| {
        let mut text_view = app.find_name::<cursive::views::TextView>("contents").unwrap();
        let content = if entry.metadata().unwrap().is_dir() { "A folder is selected".to_string() } else {
            let mut buf = String::new();
            let _ = std::fs::File::open(entry.file_name())
                .and_then(|mut f| std::io::Read::read_to_string(&mut f, &mut buf))
                .map_err(|e| buf = format!("Error: {}", e));
            buf
        };
        text_view.set_content(content);
    })
}

fn main_layout_wrap<V: cursive::view::IntoBoxedView + 'static>(view: V) -> cursive::views::Dialog {
    use cursive::traits::{Nameable, Boxable, Scrollable};
    let mut layout = cursive::views::LinearLayout::vertical();
    layout.add_child(view);
    layout.add_child(cursive::views::TextView::new("Pick an .mpa file to run")
        // .scrollable()
        .with_name("status")
        .fixed_size((80, 1)));
    cursive::views::Dialog::around(layout).button("Quit", |a| a.quit())
}

struct MenuEntry { name: String, action: fn(app: &mut cursive::Cursive) -> (), description: String }
fn main_menu() -> cursive::views::SelectView<MenuEntry> {
    let mut view = cursive::views::SelectView::new();
    let entries = [
        MenuEntry{name: "Run".to_string(), action: |a| { }, description: "Pick an .mpa file to run".into() },
        MenuEntry{name: "Edit".into(), action: |a| { a.pop_layer(); a.add_layer(main_layout_wrap(edit_menu())); }, description: "Edit the script in a real IDE, with autocompletion, breakpoints, etc".into() },
        // MenuEntry{name: "Quit".into(), action: |a| a.quit() },
    ];
    for entry in entries { view.add_item(entry.name.clone(), entry); }
    view.on_select(menu_item_highlighted).on_submit(menu_item_chosen)
}

fn edit_menu() -> cursive::views::SelectView<MenuEntry>
{
    let mut view = cursive::views::SelectView::new();
    let entries = [
        MenuEntry{name: "neovim + lspclient".into(), action: |a| {}, description: "descr".into() },
        MenuEntry{name: "vscode".into(), action: |a| {}, description: "descr".into() },
        MenuEntry{name: "Jetbrains Idea".into(), action: |a| {}, description: "descr".into() },
    ];
    for entry in entries {
        view.add_item(entry.name.clone(), entry);
    }
    view.on_select(menu_item_highlighted).on_submit(menu_item_chosen)
}

fn menu_item_chosen(app: &mut cursive::Cursive, entry: &MenuEntry) {
    (entry.action)(app);
}

fn menu_item_highlighted(app: &mut cursive::Cursive, entry: &MenuEntry) {
    let mut status_bar = app.find_name::<cursive::views::TextView>("status").unwrap();
    status_bar.set_content(&entry.description);
}