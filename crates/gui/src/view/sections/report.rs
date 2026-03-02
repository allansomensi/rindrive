use crate::app::{App, UsbHardwareInfo};
use crate::message::Message;
use iced::widget::{Space, column, container, row, text};
use iced::{Element, Length, Theme};
use rindrive_core::engine::EngineType;
use rindrive_i18n::fl;

pub fn view(app: &App, info: &UsbHardwareInfo) -> Element<'static, Message> {
    let speed_status = match info.speed {
        Some(nusb::Speed::SuperPlus) | Some(nusb::Speed::Super) => 1,
        Some(nusb::Speed::High) => 2,
        _ => 3,
    };

    let (speed_name, speed_desc) = match info.speed {
        Some(nusb::Speed::SuperPlus) => (
            fl!("report-usb-speed-super-plus"),
            fl!("report-usb-desc-super-plus"),
        ),
        Some(nusb::Speed::Super) => (fl!("report-usb-speed-super"), fl!("report-usb-desc-super")),
        Some(nusb::Speed::High) => (fl!("report-usb-speed-high"), fl!("report-usb-desc-high")),
        _ => (
            fl!("report-usb-speed-unknown"),
            fl!("report-usb-desc-unknown"),
        ),
    };

    let engine_details = match app.selected_engine {
        EngineType::SpotCheck => fl!(
            "report-engine-spotcheck",
            sections = app.sections_input.clone(),
            buffer = app.buffer_size_input.clone()
        ),
        EngineType::FullScan => fl!("report-engine-fullscan"),
    };

    let real_gb = app
        .last_report
        .as_ref()
        .map_or(0.0, |r| r.validated_size_bytes as f64 / 1_000_000_000.0);
    let size_val = format!("{real_gb:.2}");

    let has_fake_capacity = app.last_report.as_ref().is_some_and(|r| r.has_errors);

    let real_size_text = if app.last_report.is_some() {
        if has_fake_capacity {
            fl!("report-capacity-fake", size = size_val)
        } else {
            fl!("report-capacity-genuine", size = size_val)
        }
    } else {
        app.drive_capacity.clone()
    };

    let small_info = |icon: &'static str,
                      label: String,
                      value: String,
                      is_accent: bool,
                      custom_color_state: Option<u8>| {
        column![
            row![
                text(icon).size(12),
                text(label).size(11).style(text::secondary)
            ]
            .spacing(5),
            text(value).size(13).style(move |theme: &Theme| {
                let ext = theme.extended_palette();
                let color = match custom_color_state {
                    Some(1) => ext.danger.strong.color,
                    Some(2) => ext.success.strong.color,
                    _ if is_accent => ext.primary.strong.color,
                    _ => theme.palette().text,
                };
                text::Style { color: Some(color) }
            })
        ]
        .spacing(1)
    };

    container(
        column![
            text(fl!("report-title"))
                .size(14)
                .font(iced::font::Font::MONOSPACE)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().primary.strong.color)
                }),
            Space::new().height(10),
            row![
                small_info(
                    "🏷️",
                    fl!("report-label-manufacturer"),
                    info.manufacturer.clone().unwrap_or(fl!("report-value-na")),
                    false,
                    None
                )
                .width(Length::FillPortion(1)),
                small_info(
                    "📦",
                    fl!("report-label-product"),
                    info.product.clone().unwrap_or(fl!("report-value-generic")),
                    false,
                    None
                )
                .width(Length::FillPortion(1)),
            ]
            .spacing(10),
            Space::new().height(10),
            small_info(
                "💾",
                fl!("report-label-capacity"),
                real_size_text,
                false,
                if app.last_report.is_some() {
                    Some(if has_fake_capacity { 1 } else { 2 })
                } else {
                    None
                }
            ),
            Space::new().height(10),
            small_info("⚙️", fl!("report-label-engine"), engine_details, true, None),
            Space::new().height(10),
            small_info(
                "📅",
                fl!("report-label-date"),
                app.audit_time
                    .clone()
                    .unwrap_or(fl!("report-value-unknown")),
                false,
                None
            ),
            Space::new().height(15),
            container(
                column![
                    text(fl!("report-usb-title"))
                        .size(10)
                        .font(iced::font::Font::MONOSPACE)
                        .style(text::secondary),
                    text(speed_name).size(14).style(move |theme: &Theme| {
                        let ext = theme.extended_palette();
                        let color = match speed_status {
                            1 => ext.success.strong.color,
                            2 => ext.secondary.strong.color,
                            _ => ext.background.strong.color,
                        };
                        text::Style { color: Some(color) }
                    }),
                    text(speed_desc).size(10).style(move |theme: &Theme| {
                        let ext = theme.extended_palette();
                        let color = match speed_status {
                            1 => ext.success.strong.color,
                            2 => ext.secondary.strong.color,
                            _ => ext.background.strong.color,
                        };
                        text::Style { color: Some(color) }
                    }),
                ]
                .spacing(2)
            )
            .padding(10)
            .width(Length::Fill)
            .style(move |theme: &Theme| {
                let ext = theme.extended_palette();
                let speed_color = match speed_status {
                    1 => ext.success.strong.color,
                    2 => ext.secondary.strong.color,
                    _ => ext.background.strong.color,
                };
                container::Style {
                    background: Some(speed_color.scale_alpha(0.1).into()),
                    border: iced::Border {
                        color: speed_color.scale_alpha(0.5),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            }),
        ]
        .spacing(5),
    )
    .padding(15)
    .height(Length::Fill)
    .style(|theme: &Theme| container::Style {
        background: Some(theme.extended_palette().background.weak.color.into()),
        border: iced::Border {
            color: theme.extended_palette().background.strong.color,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}
