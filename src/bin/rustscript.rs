//! Copyright © Marko Vujnovic, GNU Affero General Public License v3

use rustscript::*;

#[derive(Clone)]
struct MenuFolder { name: String, description: String, children: Vec<MenuChild> }
#[derive(Clone)]
struct MenuButton { name: String, description: String, action: fn(app: &mut cursive::Cursive) -> () }

trait HasAName { fn name(&self) -> &str; }
trait HasADescription { fn description(&self) -> &str; }
trait Clickable { fn on_click(&self, app: &mut cursive::Cursive) -> (); }

#[derive(Clone)]
enum MenuChild { A(MenuFolder), B(MenuButton) }
impl HasAName for MenuChild { fn name(&self) -> &str {
    match self {
        MenuChild::A(folder) => &folder.name,
        MenuChild::B(button) => &button.name,
    }
}}

impl HasADescription for MenuChild { fn description(&self) -> &str {
    match self {
        MenuChild::A(folder) => &folder.description,
        MenuChild::B(button) => &button.description,
    }
}}

impl Clickable for MenuChild { fn on_click(&self, app: &mut cursive::Cursive) -> () {
    match self {
        MenuChild::A(folder) => { app.pop_layer(); app.add_layer(main_layout_wrap(to_tui_view(folder))); },
        MenuChild::B(button) => (button.action)(app),
    }
}}

fn display_the_tui(ui: &MenuFolder) {
    use cursive::traits::{Nameable, Boxable, Scrollable};
    use cursive::{ align::Align, };

    let mut app = cursive::default();
    let mut panes = cursive::views::LinearLayout::horizontal();
    let mut program_descr = cursive::views::LinearLayout::vertical();
    let left_pane = to_tui_view(&ui);
    panes.add_child(cursive::traits::Boxable::fixed_size(left_pane, (30, 25)));
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

fn to_tui_view(menu: &MenuFolder) -> cursive::views::SelectView<MenuChild> {
    let mut view: cursive::views::SelectView<MenuChild> = cursive::views::SelectView::new();
    for entry in &menu.children { view.add_item(entry.name().clone(), (*entry).clone()); }
    view
        .on_select(|app: &mut cursive::Cursive, entry: &MenuChild| {
            let mut status_bar = app.find_name::<cursive::views::TextView>("status").unwrap();
            status_bar.set_content(entry.description());
        })
        .on_submit(|app: &mut cursive::Cursive, entry: &MenuChild| {
            entry.on_click(app);
        })
}

fn main() -> core::result::Result<(), std::io::Error> { tokio::runtime::Runtime::new().unwrap().block_on(async {
    let main_menu_ = MenuFolder{name: "Main menu".into(), description: "".into(), children: vec![
        MenuChild::A(MenuFolder{name: "Run".to_string(), description: "Pick an .mpa file to run".into(), children: vec![] }),
        MenuChild::A(MenuFolder{name: "Edit".into(), description: "Edit the script in a real IDE, with autocompletion, breakpoints, etc".into(), children: vec![
            MenuChild::B(MenuButton{name: "neovim + lspclient".into(), action: |a| {}, description: "descr".into() }),
            MenuChild::B(MenuButton{name: "vscode".into(), action: |a| { a.quit() }, description: "descr".into() }),
            MenuChild::B(MenuButton{name: "Jetbrains Idea".into(), action: |a| {}, description: "descr".into() }),
        ]}),
    ]};

    if std::env::args().len() == 1 { display_the_tui(&main_menu_); return Ok(()); }
    let (options, positional_args) = parse_args(std::env::args());
    let script_path = &positional_args[1];
    main_(&script_path).await?;

    Ok(())
})}

pub fn file_picker<D>(directory: D) -> cursive::views::SelectView<std::fs::DirEntry> where D: AsRef<std::path::Path> {
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

