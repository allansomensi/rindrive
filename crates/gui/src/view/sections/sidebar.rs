use crate::app::App;
use crate::message::Message;
use crate::state::AppState;
use crate::view::components::{badge, input, progress};
use iced::widget::{Space, button, column, container, pick_list, progress_bar, row, text};
use iced::{Alignment, Element, Length, Theme};
use rindrive_core::engine::EngineType;
use rindrive_i18n::fl;

pub fn view(app: &App) -> Element<'static, Message> {
    let is_running = matches!(app.state, AppState::Auditing | AppState::Cancelling);
    let is_spotcheck = app.selected_engine == EngineType::SpotCheck;

    let header = column![
        text(fl!("app-title"))
            .size(30)
            .font(iced::font::Font::MONOSPACE),
        text(fl!("app-subtitle")).size(12).style(text::secondary),
        Space::new().height(8),
        badge::status(&app.log),
    ]
    .spacing(2);

    let mut settings_col = column![
        text(fl!("settings-title"))
            .size(11)
            .font(iced::font::Font::MONOSPACE)
            .style(text::secondary),
    ];

    if is_spotcheck {
        settings_col = settings_col.push(
            row![
                input::group(
                    fl!("settings-sections-label"),
                    &app.sections_input,
                    Message::SectionsChanged,
                    !is_running,
                ),
                input::group(
                    fl!("settings-buffer-label"),
                    &app.buffer_size_input,
                    Message::BufferSizeChanged,
                    !is_running,
                ),
            ]
            .spacing(10),
        );
    }

    let engine_selector: Element<_> = if is_running {
        container(
            text(match app.selected_engine {
                EngineType::SpotCheck => "SpotCheck",
                EngineType::FullScan => "FullScan",
            })
            .size(13)
            .style(text::secondary),
        )
        .padding(6)
        .width(Length::Fill)
        .into()
    } else {
        pick_list(
            &[EngineType::SpotCheck, EngineType::FullScan][..],
            Some(app.selected_engine),
            Message::EngineSelected,
        )
        .text_size(13)
        .padding(6)
        .width(Length::Fill)
        .into()
    };

    let settings_content = settings_col
        .push(
            column![
                text(fl!("settings-engine-label"))
                    .size(11)
                    .style(text::secondary),
                engine_selector
            ]
            .spacing(4),
        )
        .spacing(12);

    let settings_panel = container(settings_content)
        .padding(15)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.weak.color.into()),
                border: iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

    let drive_section: Element<_> = if let Some(drive_path) = &app.selected_drive {
        let mount_point = drive_path.display().to_string();

        let info_col = column![
            text(app.drive_name.clone()).size(13),
            text(format!("{} • {mount_point}", app.drive_capacity))
                .size(11)
                .style(text::secondary)
        ]
        .spacing(2)
        .width(Length::Fill);

        let mut btn_change = button(text("🔄").size(14).center())
            .style(button::secondary)
            .padding(6);

        if !is_running {
            btn_change = btn_change.on_press(Message::UnselectDrive);
        }

        container(
            row![text("💾").size(18), info_col, btn_change]
                .spacing(10)
                .align_y(Alignment::Center),
        )
        .padding(10)
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.weak.color.into()),
                border: iced::Border {
                    radius: 6.0.into(),
                    color: if is_running {
                        palette.background.weak.color
                    } else {
                        palette.primary.strong.color.scale_alpha(0.5)
                    },
                    width: 1.0,
                },
                ..Default::default()
            }
        })
        .into()
    } else {
        let mut btn = button(text(fl!("btn-select-drive")).size(14).center())
            .style(button::secondary)
            .padding(12)
            .width(Length::Fill);

        if !is_running {
            btn = btn.on_press(Message::SelectManual);
        }
        btn.into()
    };

    let start_btn = match app.state {
        AppState::Auditing => button(
            text(fl!("btn-cancel-audit"))
                .size(14)
                .font(iced::font::Font::MONOSPACE)
                .center(),
        )
        .style(button::danger)
        .padding(12)
        .width(Length::Fill)
        .on_press(Message::CancelAudit),

        AppState::Cancelling => button(
            text(fl!("btn-cancelling"))
                .size(14)
                .font(iced::font::Font::MONOSPACE)
                .center(),
        )
        .style(button::secondary)
        .padding(12)
        .width(Length::Fill),

        _ => {
            let btn = button(
                text(fl!("btn-start-audit"))
                    .size(14)
                    .font(iced::font::Font::MONOSPACE)
                    .center(),
            )
            .style(button::primary)
            .padding(12)
            .width(Length::Fill);

            if matches!(app.state, AppState::Ready | AppState::Finished) {
                btn.on_press(Message::StartAudit)
            } else {
                btn
            }
        }
    };

    let progress_section = column![
        row![
            text(fl!("progress-title"))
                .size(11)
                .font(iced::font::Font::MONOSPACE)
                .style(text::secondary),
            Space::width(Space::new(), Length::Fill),
            text(format!("{:.1}%", app.progress))
                .size(12)
                .style(text::secondary)
        ],
        Space::new().height(4),
        container(progress_bar(0.0..=100.0, app.progress).style(progress::style)).height(6)
    ];

    let controls_panel = column![
        Space::new().height(8),
        progress_section,
        Space::new().height(15),
        drive_section,
        Space::new().height(8),
        start_btn
    ];

    container(column![
        header,
        Space::new().height(20),
        settings_panel,
        Space::width(Space::new(), Length::Fill),
        controls_panel
    ])
    .width(Length::Fixed(320.0))
    .height(Length::Fill)
    .padding(20)
    .style(|theme: &Theme| container::Style {
        background: Some(theme.extended_palette().background.base.color.into()),
        ..Default::default()
    })
    .into()
}
