//! Copyright © Marko Vujnovic, GNU Affero General Public License v3

use rustscript::*;

#[derive(Clone)]
pub struct MenuFolder { name: String, description: String, children: Vec<EMenuChild> }
#[derive(Clone)]
pub struct MenuButton { name: String, description: String, action: fn(app: &mut cursive::Cursive) -> () }

trait HasAName { fn name(&self) -> &str; }
trait HasADescription { fn description(&self) -> &str; }
trait Clickable { fn on_click(&self, app: &mut cursive::Cursive) -> (); }

#[derive(Clone)]
pub enum EMenuChild { A(MenuFolder), B(MenuButton) }
impl HasAName for EMenuChild { fn name(&self) -> &str {
    match self {
        EMenuChild::A(folder) => &folder.name,
        EMenuChild::B(button) => &button.name,
    }
}}

impl HasADescription for EMenuChild { fn description(&self) -> &str {
    match self {
        EMenuChild::A(folder) => &folder.description,
        EMenuChild::B(button) => &button.description,
    }
}}

impl Clickable for EMenuChild { fn on_click(&self, app: &mut cursive::Cursive) -> () {
    match self {
        EMenuChild::A(folder) => { app.pop_layer(); app.add_layer(main_layout_wrap(to_tui_view(folder))); },
        EMenuChild::B(button) => (button.action)(app),
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
    // while app.is_running() { app.step(); }
    // app.set_on_post_event(trigger, cb)
}

fn to_tui_view(menu: &MenuFolder) -> cursive::views::SelectView<EMenuChild> {
    let mut view: cursive::views::SelectView<EMenuChild> = cursive::views::SelectView::new();
    for entry in &menu.children { view.add_item(entry.name().clone(), (*entry).clone()); }
    view
        .on_select(|app: &mut cursive::Cursive, entry: &EMenuChild| {
            let mut status_bar = app.find_name::<cursive::views::TextView>("status").unwrap();
            status_bar.set_content(entry.description());
        })
        .on_submit(|app: &mut cursive::Cursive, entry: &EMenuChild| {
            entry.on_click(app);
        })
}

fn await_ui<V: cursive::view::IntoBoxedView + 'static>(view: V, a: &mut cursive::Cursive,) {
    a.pop_layer();
    a.add_layer(main_layout_wrap(view));
}

fn await_blocking<F: core::future::Future>(future: F) -> F::Output {
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    tokio_runtime.block_on(future)
}

fn main() -> core::result::Result<(), std::io::Error> {

    let main_menu_ = MenuFolder{name: "Main menu".into(), description: "".into(), children: vec![
        EMenuChild::B(MenuButton{name: "Run".to_string(), description: "Pick an .mpa file to run".into(), action: |a| {
            // let file_or_folder = file_picker(a, ".").await;
            await_ui(file_picker(".", |file_or_folder: &std::path::Path, a| {
            await_blocking(main_(file_or_folder.to_str().unwrap(), true)).unwrap();
            a.quit();
        }), a); } }),
        EMenuChild::B(MenuButton{name: "Edit".into(), description: "Edit the script in a real IDE, with autocompletion, breakpoints, etc".into(), action: |a| {
            await_ui(file_picker(".", |file_or_folder: &std::path::Path, a| {
            await_ui(ide_picker(|ide: &IDE, a: &mut cursive::Cursive, file_or_folder: &std::path::Path| {
            await_blocking(edit_(&file_or_folder, &ide)).unwrap();
        }, file_or_folder), a); }), a); }}),
    ]};

    if std::env::args().len() == 1 { display_the_tui(&main_menu_); return Ok(()); }
    let (options, positional_args) = parse_args(std::env::args());
    let script_path = &positional_args[1];
    await_blocking(main_(&script_path, false))
}

// pub async fn file_picker<D: AsRef<std::path::Path>>(app: &mut cursive::Cursive, directory: D) -> core::result::Result<bool, std::io::Error> {
pub fn file_picker<D, F: 'static + Fn(&std::path::Path, &mut cursive::Cursive) -> ()>(directory: D, on_picked: F) -> cursive::views::SelectView<std::fs::DirEntry> where D: AsRef<std::path::Path> {
    let mut view = cursive::views::SelectView::new();
    for entry in std::fs::read_dir(directory).expect("Can't read the directory") {
        if let Ok(e) = entry {
            let file_name = e.file_name().into_string().unwrap();
            view.add_item(file_name, e);
        }
    }
    view
    .on_select(|app: &mut cursive::Cursive, entry: &std::fs::DirEntry| {
        let mut status_bar = app.find_name::<cursive::views::TextView>("status").unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        let file_size = entry.metadata().unwrap().len();
        let content = format!("{}: {} bytes", file_name, file_size);
        status_bar.set_content(content);
    })
    .on_submit(move |app: &mut cursive::Cursive, entry: &std::fs::DirEntry| {
        if entry.metadata().unwrap().is_dir() {
            // list folder's files
        } else {
            on_picked(&entry.path(), app);
        };
    })
}

pub fn ide_picker<F: 'static + Clone + Fn(&IDE, &mut cursive::Cursive, &std::path::Path) -> ()>(on_picked: F, captured: &std::path::Path) -> cursive::views::SelectView<(EMenuChild, std::path::PathBuf, F)> {
    let ides = vec![
        // EMenuChild::B(MenuButton{name: "neovim + lspclient".into(), action: |a| {}, description: "descr".into() }),
        EMenuChild::B(MenuButton{name: "Your local vscode".into(), action: |a| {}, description: "descr".into() }),
        EMenuChild::B(MenuButton{name: "Your local neovim".into(), action: |a| {}, description: "descr".into() }),
        // EMenuChild::B(MenuButton{name: "Marko's vscode".into(), action: |a| {}, description: "descr".into() }),
        // EMenuChild::B(MenuButton{name: "Jetbrains Idea".into(), action: |a| {}, description: "descr".into() }),
    ];

    let mut view = cursive::views::SelectView::new();
    for ide in &ides {
        let label = ide.name();
        let obj_to_store = (ide.clone(), captured.to_path_buf(), on_picked.clone());
        view.add_item(label, obj_to_store);
    }
    view
    .on_submit(|a, stored_obj| { 
        if stored_obj.0.name() == "Your local vscode" { (stored_obj.2)(&IDE::VSCode_(VSCode{}), a, &stored_obj.1); }
        else if stored_obj.0.name() == "Your local neovim" { (stored_obj.2)(&IDE::Vim_(Vim{}), a, &stored_obj.1); }
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

