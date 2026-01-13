use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    symbols,
    text::Line,
    widgets::{Block, Borders, StatefulWidget, Widget},
};
use tui_prompts::{State, TextPrompt, TextState};

use crate::engine::{self, BackupRequest, EngineControl};

pub struct CreateBackupView<'a> {
    engine_control: EngineControl,
    backup_name: TextState<'a>,
    is_done: bool,
}

impl CreateBackupView<'_> {
    pub fn new(engine_control: EngineControl) -> Self {
        let backup_name = TextState::new();

        Self {
            engine_control,
            backup_name,
            is_done: false,
        }
    }

    pub fn on_key_event(&mut self, event: KeyEvent) -> anyhow::Result<()> {
        match event.code {
            KeyCode::Esc => self.is_done = true,
            KeyCode::Enter => {
                self.backup_name.complete();
                self.create_backup()?;
                return Ok(());
            }
            KeyCode::Down | KeyCode::Up => {}
            _ => {}
        }

        self.backup_name.handle_key_event(event);

        Ok(())
    }

    pub fn is_done(&self) -> bool {
        self.is_done
    }

    pub fn create_backup(&mut self) -> anyhow::Result<()> {
        if self.is_done {
            return Ok(());
        }

        self.is_done = true;

        let description = self.backup_name.value();

        if description.is_empty() {
            return Ok(());
        }

        let archive_name = engine::make_backup_filename(description);

        self.engine_control.send(BackupRequest::CreateBackup { archive_name })?;

        Ok(())
    }
}

impl Widget for &mut CreateBackupView<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::raw("Create backup");

        let block = Block::default()
            .title(title)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default())
            .borders(Borders::all());

        let backup_name = TextPrompt::from("Backup name").with_block(block);

        let [backup_name_area, _] = Layout::vertical([Constraint::Length(3), Constraint::Length(10)]).areas(area);

        backup_name.render(backup_name_area, buf, &mut self.backup_name);
    }
}
