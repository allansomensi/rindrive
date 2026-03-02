use iced::Theme;
use iced::widget::progress_bar;

pub fn style(theme: &Theme) -> progress_bar::Style {
    let ext = theme.extended_palette();
    progress_bar::Style {
        background: ext.background.strong.color.into(),
        bar: ext.primary.strong.color.into(),
        border: iced::Border::default(),
    }
}
