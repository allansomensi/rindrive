use crate::app::App;
use crate::message::Message;
use crate::state::AppState;
use iced::widget::{
    Space, button, canvas, column, container, pick_list, progress_bar, row, text, text_input,
};
use iced::{Alignment, Color, Element, Length, Point, Rectangle, Size, Theme, mouse};
use rindrive_core::engine::EngineType;
use rindrive_i18n::fl;

const COLOR_BG_SIDEBAR: Color = Color::from_rgb(0.11, 0.11, 0.11);
const COLOR_BG_MAP: Color = Color::from_rgb(0.05, 0.05, 0.05);
const COLOR_ACCENT: Color = Color::from_rgb(0.25, 0.65, 1.0);
const COLOR_VALID: Color = Color::from_rgb(0.0, 0.8, 0.4);
const COLOR_INVALID: Color = Color::from_rgb(0.9, 0.25, 0.25);
const COLOR_PENDING: Color = Color::from_rgb(0.15, 0.15, 0.15);
const COLOR_READING: Color = Color::from_rgb(0.2, 0.6, 1.0);
const COLOR_WRITING: Color = Color::from_rgb(0.9, 0.6, 0.1);

pub fn view(app: &App) -> Element<'static, Message> {
    let is_running = matches!(app.state, AppState::Auditing | AppState::Cancelling);
    let is_spotcheck = app.selected_engine == EngineType::SpotCheck;

    let header = column![
        text(fl!("app-title"))
            .size(30)
            .font(iced::font::Font::MONOSPACE)
            .style(|_| text::Style {
                color: Some(Color::WHITE)
            }),
        text(fl!("app-subtitle")).size(12).style(text::secondary),
        Space::new().height(8),
        view_status_badge(&app.log),
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
                input_group(
                    fl!("settings-sections-label"),
                    &app.sections_input,
                    Message::SectionsChanged,
                    !is_running,
                ),
                input_group(
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
            text(app.drive_name.clone())
                .size(13)
                .style(|_| text::Style {
                    color: Some(Color::WHITE)
                }),
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
                        COLOR_ACCENT.scale_alpha(0.5)
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

    let mut start_btn = match app.state {
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

    if matches!(app.state, AppState::Ready | AppState::Finished) {
        start_btn = start_btn.on_press(Message::StartAudit);
    }

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
        container(progress_bar(0.0..=100.0, app.progress).style(style_progress_bar)).height(6)
    ];

    let controls_panel = column![
        Space::new().height(8),
        progress_section,
        Space::new().height(15),
        drive_section,
        Space::new().height(8),
        start_btn
    ];

    let sidebar = container(column![
        header,
        Space::new().height(20),
        settings_panel,
        Space::width(Space::new(), Length::Fill),
        controls_panel
    ])
    .width(Length::Fixed(320.0))
    .height(Length::Fill)
    .padding(20)
    .style(|_| container::Style {
        background: Some(COLOR_BG_SIDEBAR.into()),
        ..Default::default()
    });

    let map_legend = row![
        legend_badge(COLOR_READING, fl!("legend-read")),
        legend_badge(COLOR_WRITING, fl!("legend-write")),
        legend_badge(COLOR_VALID, fl!("legend-valid")),
        legend_badge(COLOR_INVALID, fl!("legend-bad")),
    ]
    .spacing(15);

    let map_area = container(
        canvas(BlockMap {
            blocks: app.block_map.clone(),
        })
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(10)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(COLOR_BG_MAP.into()),
            border: iced::Border {
                color: palette.background.strong.color,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    });

    let main_content = column![
        container(map_legend)
            .padding([10, 20])
            .align_x(Alignment::Center)
            .width(Length::Fill),
        map_area
    ];

    row![sidebar, main_content]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn input_group<'a>(
    label: String,
    value: &str,
    on_change: fn(String) -> Message,
    enabled: bool,
) -> Element<'a, Message> {
    let mut input = text_input("", value)
        .padding(8)
        .size(14)
        .style(|theme, status| {
            let mut style = text_input::default(theme, status);
            style.border.radius = 4.0.into();
            style
        });

    if enabled {
        input = input.on_input(on_change);
    }

    column![text(label).size(12).style(text::secondary), input]
        .spacing(4)
        .width(Length::Fill)
        .into()
}

fn legend_badge<'a>(color: Color, label: String) -> Element<'a, Message> {
    row![
        container(Space::new())
            .width(12)
            .height(12)
            .style(move |_| {
                container::Style {
                    background: Some(color.into()),
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }),
        text(label).size(12).style(text::secondary)
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

fn view_status_badge(log: &str) -> Element<'static, Message> {
    let (icon, color): (&str, Color) = if log.contains("❌") {
        ("⚠️", COLOR_INVALID)
    } else if log.contains("✅") {
        ("🛡️", COLOR_VALID)
    } else {
        ("ℹ️", COLOR_ACCENT)
    };

    container(
        row![
            text(icon).size(16),
            text(log.to_string())
                .size(12)
                .style(move |_| text::Style { color: Some(color) })
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([6, 10])
    .style(move |_| container::Style {
        background: Some(color.scale_alpha(0.08).into()),
        border: iced::Border {
            color: color.scale_alpha(0.2),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn style_progress_bar(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: COLOR_PENDING.into(),
        bar: COLOR_ACCENT.into(),
        border: iced::Border::default(),
    }
}

struct BlockMap {
    blocks: Vec<u8>,
}

impl canvas::Program<Message> for BlockMap {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let total = self.blocks.len();

        if total == 0 {
            return vec![];
        }

        let screen_ratio = bounds.width / bounds.height;
        let rows = (total as f32 / screen_ratio).sqrt().ceil();
        let mut cols = (total as f32 / rows).ceil();

        while (rows * cols) < total as f32 {
            cols += 1.0;
        }

        let w_space = bounds.width / cols;
        let h_space = bounds.height / rows;
        let box_size = w_space.min(h_space);

        let spacing = if box_size < 3.0 { 0.0 } else { 1.0 };
        let draw_size = (box_size - spacing).max(1.0);

        let grid_width = cols * box_size;
        let grid_height = rows * box_size;
        let start_x = (bounds.width - grid_width) / 2.0;
        let start_y = (bounds.height - grid_height) / 2.0;

        for (i, &status) in self.blocks.iter().enumerate() {
            let col = i as f32 % cols;
            let row = (i as f32 / cols).floor();

            let x = start_x + (col * box_size);
            let y = start_y + (row * box_size);

            if y > bounds.height {
                break;
            }

            let color = match status {
                1 => COLOR_VALID,
                2 => COLOR_INVALID,
                3 => COLOR_READING,
                4 => COLOR_WRITING,
                _ => COLOR_PENDING,
            };

            frame.fill_rectangle(Point::new(x, y), Size::new(draw_size, draw_size), color);
        }

        vec![frame.into_geometry()]
    }
}
