//mod uihandler;

use iocraft::prelude::*;
use std::time::Duration;

//pub use self::uihandler::TuiUiHandler;

#[component]
pub fn Example(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut time = hooks.use_state(|| time::UtcDateTime::now());
    let mut should_exit = hooks.use_state(|| false);
    let mut autobackup = hooks.use_state(|| true);
    let mut log_entries = hooks.use_state(|| Vec::<String>::new());

    hooks.use_future(async move {
        loop {
            smol::Timer::after(Duration::from_secs(1)).await;
            time.set(time::UtcDateTime::now());
            log_entries.write().push("TEST LOG TEXT".to_owned());
        }
    });

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => match code {
                KeyCode::Char('q') => should_exit.set(true),
                KeyCode::F(12) => autobackup.set(!autobackup.get()),
                _ => {}
            },
            _ => {}
        }
    });

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
            View(
                border_style: BorderStyle::Round,
                border_color: Color::Blue,
                margin_bottom: 0,
                padding_top: 2,
                padding_bottom: 2,
                padding_left: 8,
                padding_right: 8,
                flex_grow: 1.0f32,
            ) {
                Text(content: format!("Current Time: {}", time.get().to_string()))
            }
            View(
                border_style: BorderStyle::Round,
                border_color: Color::Blue,
                flex_direction: FlexDirection::Column,
                height: 10,
            ) {
                #(log_entries.read().iter().map(|e| {
                    element! {
                        Text(content: format!("{e}"))
                    }
                }))
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
