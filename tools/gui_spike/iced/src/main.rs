#![windows_subsystem = "windows"]

use gui_spike_common as common;
use iced::widget::{button, column, container, progress_bar, row, rule, text};
use iced::{Element, Length, Size, Subscription, Task, window};

struct SpikeApp {
    shows: Vec<(String, String)>,
    selected: Option<usize>,
    progress: f32,
    frames: u32,
}

#[derive(Debug, Clone)]
enum Message {
    Select(usize),
    Add,
    Remove,
    Run,
    OpenPdf,
    Frame,
    Shot(window::Screenshot),
}

fn boot() -> SpikeApp {
    SpikeApp { shows: common::shows(), selected: Some(0), progress: 0.45, frames: 0 }
}

fn subscription(_app: &SpikeApp) -> Subscription<Message> {
    match common::screenshot_arg() {
        Some(_) => window::frames().map(|_| Message::Frame),
        None => Subscription::none(),
    }
}

fn update(app: &mut SpikeApp, message: Message) -> Task<Message> {
    match message {
        Message::Select(index) => app.selected = Some(index),
        Message::Add => {
            let (name, publisher) = common::ADDED_SHOW;
            app.shows.push((name.to_string(), publisher.to_string()));
        }
        Message::Remove => {
            if let Some(index) = app.selected.take() {
                app.shows.remove(index);
            }
        }
        Message::Run => app.progress = common::advance(app.progress),
        Message::OpenPdf => common::open_pdf(),
        Message::Frame => {
            app.frames += 1;
            if app.frames == 3 {
                return window::latest().and_then(window::screenshot).map(Message::Shot);
            }
        }
        Message::Shot(shot) => {
            if let Some(path) = common::screenshot_arg() {
                common::save_png(&path, shot.size.width, shot.size.height, &shot.rgba);
            }
            return iced::exit();
        }
    }
    Task::none()
}

fn view(app: &SpikeApp) -> Element<'_, Message> {
    let shows = app.shows.iter().enumerate().fold(column![].spacing(2), |list, (index, (name, publisher))| {
        let style = if app.selected == Some(index) { button::primary } else { button::text };
        list.push(button(text(format!("{name}  ·  {publisher}"))).width(Length::Fill).style(style).on_press(Message::Select(index)))
    });
    let remove = button(common::REMOVE).on_press_maybe(app.selected.map(|_| Message::Remove));
    column![
        text(common::SHOWS_HEADING).size(22),
        container(shows).padding(6).width(Length::Fill).style(container::bordered_box),
        row![button(common::ADD).on_press(Message::Add), remove].spacing(8),
        rule::horizontal(1),
        text(common::PROGRESS_HEADING).size(22),
        text(common::status(app.progress)),
        progress_bar(0.0..=1.0, app.progress),
        row![button(common::RUN).on_press(Message::Run), button(common::OPEN_PDF).on_press(Message::OpenPdf)].spacing(8),
        rule::horizontal(1),
        text(common::ACCENTS),
    ]
    .spacing(10)
    .padding(12)
    .into()
}

fn main() -> iced::Result {
    let (width, height) = common::WINDOW_SIZE;
    iced::application(boot, update, view)
        .title(common::WINDOW_TITLE)
        .subscription(subscription)
        .window_size(Size::new(width, height))
        .run()
}
