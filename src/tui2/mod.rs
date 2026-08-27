//mod uihandler;

use std::{sync::Arc, time::Duration};

use iocraft::prelude::*;

//pub use self::uihandler::TuiUiHandler;

#[derive(Clone, Copy, PartialEq)]
pub enum MenuAction {
    CreateBackup,
    RestoreBackup,
    Exit,
}

struct MenuChoice<T> {
    pub label: String,
    pub action: T,
}

#[component]
pub fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut should_exit = hooks.use_state(|| false);
    let mut autobackup = hooks.use_state(|| true);
    let mut log_entries = hooks.use_state(|| Vec::<String>::new());
    let mut menu_selection = hooks.use_state(|| 0usize);
    let mut menu_action = hooks.use_state::<Option<MenuAction>, _>(|| None);

    let menu_items = hooks.use_const(|| {
        Arc::new(vec![
            MenuChoice {
                label: "Create backup".to_owned(),
                action: MenuAction::CreateBackup,
            },
            MenuChoice {
                label: "Restore backup".to_owned(),
                action: MenuAction::RestoreBackup,
            },
            MenuChoice {
                label: "Exit".to_owned(),
                action: MenuAction::Exit,
            },
        ])
    });

    hooks.use_future(async move {
        loop {
            smol::Timer::after(Duration::from_secs(1)).await;
            log_entries.write().push("TEST LOG TEXT".to_owned());
        }
    });

    hooks.use_terminal_events({
        let menu_items = menu_items.clone();

        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => match code {
                KeyCode::Char('q') => should_exit.set(true),
                KeyCode::F(12) => autobackup.set(!autobackup.get()),
                KeyCode::Down => menu_selection.set(menu_move(menu_selection.get(), 1, &menu_items)),
                KeyCode::Up => menu_selection.set(menu_move(menu_selection.get(), -1, &menu_items)),
                KeyCode::Enter => menu_action.set(menu_items.get(menu_selection.get()).map(|mi| mi.action)),
                _ => {}
            },
            _ => {}
        }
    });

    if let Some(action) = menu_action.get() {
        menu_action.set(None);

        match action {
            MenuAction::CreateBackup => {
                log_entries.write().push("CREATE BACKUP".to_owned());
            }
            MenuAction::RestoreBackup => {
                log_entries.write().push("RESTORE BACKUP".to_owned());
            }
            MenuAction::Exit => should_exit.set(true),
        }
    }

    if should_exit.get() {
        system.exit();
    }

    element! {
        View(
            width,
            height,
            background_color: Color::DarkGrey,
            border_style: BorderStyle::None,
            border_color: Color::Blue,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Start,
        ) {
            // Menu
            View(
                border_style: BorderStyle::Round,
                border_color: Color::Blue,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                padding_top: 2,
                padding_bottom: 2,
                padding_left: 8,
                padding_right: 8,
                flex_grow: 1.0f32,
            ) {
                View(
                    border_style: BorderStyle::Round,
                    border_color: Color::Blue,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    padding_left: 1,
                    padding_right: 1,
                ) {
                    #(menu_items.iter().enumerate().map(|(i, item)| {
                        let cursor = if i == menu_selection.get() { ">" } else { " " };
                        let label = &item.label;

                        element! {
                            Text(content: format!("{cursor} {label}"))
                        }
                    }))
                }
            }
            // Log view
            View(
                border_style: BorderStyle::Round,
                border_color: Color::Blue,
                flex_direction: FlexDirection::Column,
                height: 10,
            ) {
                ScrollView {
                    View(
                        flex_direction: FlexDirection::Column,
                    ) {
                        #(log_entries.read().iter().map(|e| {
                            element! {
                                Text(content: format!("{e}"))
                            }
                        }))
                    }
                }
            }
            View(
                border_style: BorderStyle::None,
                border_color: Color::Blue,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Baseline,
                justify_content: JustifyContent::Start,
            ) {
                AutobackupStatus(state: autobackup.get())
            }
        }
    }
}

#[derive(Default, Props)]
struct AutobackupStatusProps {
    pub state: bool,
}

#[component]
fn AutobackupStatus(props: &AutobackupStatusProps) -> impl Into<AnyElement<'static>> {
    let (state_color, state_text) = if props.state {
        (Color::Green, "ON")
    } else {
        (Color::Red, "OFF")
    };

    element! {
        View(
            background_color: state_color,
            padding_left: 1,
            padding_right: 1,
        ) {
            Text(content: format!("Autobackup {state_text:^3}"))
        }
    }
}

fn menu_move<T>(ix: usize, step: isize, items: &[MenuChoice<T>]) -> usize {
    (ix as isize + step).rem_euclid(items.len() as isize) as usize
}
