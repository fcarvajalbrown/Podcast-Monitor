use std::path::PathBuf;

use iced::widget::{button, column, container, row, rule, scrollable, text, text_input};
use iced::{Element, Length, Size, Task};

use crate::feed;
use crate::shows::{self, SavedShow};

const TITLE: &str = "Podcast-Monitor";
const SHOWS_HEADING: &str = "Programas guardados";
const NO_SHOWS: &str = "Todavía no hay programas guardados.";
const FEED_PLACEHOLDER: &str = "Pega aquí la dirección del feed RSS";
const ADD_FEED: &str = "Agregar";
const REMOVE: &str = "Quitar";
const FETCHING: &str = "Leyendo el feed…";
const BAD_FEED: &str = "No se pudo leer ese enlace. Revisa que sea la dirección del feed RSS del programa.";
const NO_AUDIO: &str = "Ese feed no tiene episodios con audio.";
const ALREADY_SAVED: &str = "Ese programa ya está guardado.";
const LOAD_FAILED: &str = "No se pudo leer la lista de programas guardados; los cambios no se guardarán.";
const SAVE_FAILED: &str = "No se pudo guardar la lista de programas.";

struct Window {
    list_path: Option<PathBuf>,
    shows: Vec<SavedShow>,
    selected: Option<String>,
    feed_input: String,
    fetching: bool,
    notice: Option<&'static str>,
}

#[derive(Debug, Clone)]
enum Message {
    FeedInput(String),
    AddFeed,
    Fetched(Result<SavedShow, &'static str>),
    Select(String),
    Remove,
}

pub fn run() -> iced::Result {
    iced::application(boot, update, view).title(TITLE).window_size(Size::new(600.0, 460.0)).run()
}

fn boot() -> Window {
    open(shows::list_path(&shows::data_dir()))
}

fn open(path: PathBuf) -> Window {
    let (list_path, shows, notice) = match shows::load(&path) {
        Ok(shows) => (Some(path), shows, None),
        Err(error) => {
            eprintln!("loading the show list: {error:#}");
            (None, Vec::new(), Some(LOAD_FAILED))
        }
    };
    Window { list_path, shows, selected: None, feed_input: String::new(), fetching: false, notice }
}

fn fetch_show(url: String) -> Result<SavedShow, &'static str> {
    let fetched = feed::fetch(&url).map_err(|error| {
        eprintln!("adding {url}: {error:#}");
        BAD_FEED
    })?;
    if fetched.episodes.is_empty() {
        return Err(NO_AUDIO);
    }
    Ok(SavedShow { title: fetched.title, publisher: fetched.author, feed: url })
}

fn save(window: &mut Window) {
    let Some(path) = &window.list_path else { return };
    if let Err(error) = shows::save(path, &window.shows) {
        eprintln!("saving the show list: {error:#}");
        window.notice = Some(SAVE_FAILED);
    }
}

fn update(window: &mut Window, message: Message) -> Task<Message> {
    match message {
        Message::FeedInput(input) => window.feed_input = input,
        Message::AddFeed => {
            let url = window.feed_input.trim().to_string();
            if url.is_empty() || window.fetching {
                return Task::none();
            }
            if window.shows.iter().any(|saved| saved.feed == url) {
                window.notice = Some(ALREADY_SAVED);
                return Task::none();
            }
            window.fetching = true;
            window.notice = Some(FETCHING);
            return Task::perform(async move { fetch_show(url) }, Message::Fetched);
        }
        Message::Fetched(result) => {
            window.fetching = false;
            window.notice = None;
            match result {
                Ok(show) => {
                    if shows::add(&mut window.shows, show) {
                        window.feed_input.clear();
                        save(window);
                    } else {
                        window.notice = Some(ALREADY_SAVED);
                    }
                }
                Err(notice) => window.notice = Some(notice),
            }
        }
        Message::Select(feed) => window.selected = Some(feed),
        Message::Remove => {
            if let Some(feed) = window.selected.take() {
                shows::remove(&mut window.shows, &feed);
                save(window);
            }
        }
    }
    Task::none()
}

fn show_row<'a>(show: &'a SavedShow, selected: bool) -> Element<'a, Message> {
    let label = if show.publisher.is_empty() { show.title.clone() } else { format!("{}  ·  {}", show.title, show.publisher) };
    let style = if selected { button::primary } else { button::text };
    button(text(label)).width(Length::Fill).style(style).on_press(Message::Select(show.feed.clone())).into()
}

fn view(window: &Window) -> Element<'_, Message> {
    let list: Element<'_, Message> = if window.shows.is_empty() {
        text(NO_SHOWS).into()
    } else {
        let rows = window.shows.iter().map(|show| show_row(show, window.selected.as_deref() == Some(show.feed.as_str())));
        scrollable(column(rows).spacing(2)).into()
    };
    let input = text_input(FEED_PLACEHOLDER, &window.feed_input).id("feed").on_input(Message::FeedInput).on_submit(Message::AddFeed);
    let add = button(ADD_FEED).on_press_maybe((!window.fetching).then_some(Message::AddFeed));
    let remove = button(REMOVE).on_press_maybe(window.selected.as_ref().map(|_| Message::Remove));
    column![
        text(SHOWS_HEADING).size(22),
        container(list).padding(6).width(Length::Fill).height(Length::Fill).style(container::bordered_box),
        row![remove].spacing(8),
        rule::horizontal(1),
        row![input, add].spacing(8),
        text(window.notice.unwrap_or("")),
    ]
    .spacing(10)
    .padding(12)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced_test::simulator;

    const FEED: &str = "https://mdstrm.com/feeds/65ea0c645728c0087b186ba5/67f56020881515c36adbe29f?property=itunes";

    fn drive(window: &mut Window, act: impl FnOnce(&mut iced_test::Simulator<'_, Message>)) {
        let mut ui = simulator(view(window));
        act(&mut ui);
        for message in ui.into_messages().collect::<Vec<_>>() {
            let _ = update(window, message);
        }
    }

    #[test]
    fn add_by_feed_url_then_remove_survives_a_reopen() {
        let dir = std::env::temp_dir().join(format!("podcast-monitor-gui-{}", std::process::id()));
        let path = shows::list_path(&dir);
        let mut window = open(path.clone());
        assert!(simulator(view(&window)).find(NO_SHOWS).is_ok());

        drive(&mut window, |ui| {
            ui.click(iced_test::selector::id("feed")).unwrap();
            ui.typewrite(FEED);
            ui.click(ADD_FEED).unwrap();
        });
        assert_eq!(window.feed_input, FEED);
        assert!(window.fetching);
        assert_eq!(window.notice, Some(FETCHING));

        let show = SavedShow { title: "Nada Personal".into(), publisher: "Radio Duna".into(), feed: FEED.into() };
        let _ = update(&mut window, Message::Fetched(Ok(show.clone())));
        assert!(window.feed_input.is_empty());

        let mut window = open(path.clone());
        assert_eq!(window.shows, [show]);
        drive(&mut window, |ui| {
            ui.click("Nada Personal  ·  Radio Duna").unwrap();
        });
        drive(&mut window, |ui| {
            ui.click(REMOVE).unwrap();
        });
        assert!(open(path).shows.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_failed_fetch_or_a_saved_feed_shows_a_notice() {
        let dir = std::env::temp_dir().join(format!("podcast-monitor-gui-notice-{}", std::process::id()));
        let mut window = open(shows::list_path(&dir));
        let _ = update(&mut window, Message::Fetched(Err(BAD_FEED)));
        assert!(simulator(view(&window)).find(BAD_FEED).is_ok());
        let show = SavedShow { title: "Nada Personal".into(), publisher: "Radio Duna".into(), feed: FEED.into() };
        let _ = update(&mut window, Message::Fetched(Ok(show)));
        let _ = update(&mut window, Message::FeedInput(FEED.into()));
        let _ = update(&mut window, Message::AddFeed);
        assert_eq!(window.notice, Some(ALREADY_SAVED));
        assert!(!window.fetching);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    #[ignore = "fetches the Nada Personal and Animales Politicos feeds over the network"]
    fn fetches_title_and_publisher_from_real_feeds() {
        let nada = fetch_show(FEED.into()).unwrap();
        assert_eq!((nada.title.as_str(), nada.publisher.as_str()), ("Nada Personal", "Radio Duna"));
        let animales = fetch_show("https://anchor.fm/s/106ac41b0/podcast/rss".into()).unwrap();
        println!("{} | {}", animales.title, animales.publisher);
        assert!(fetch_show("https://example.com/".into()).is_err());
    }
}
