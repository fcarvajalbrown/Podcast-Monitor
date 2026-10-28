use std::path::PathBuf;

use iced::widget::{button, column, container, row, rule, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Size, Task};

use crate::directory;
use crate::feed;
use crate::shows::{self, SavedShow};

const TITLE: &str = "Podcast-Monitor";
const SHOWS_HEADING: &str = "Programas guardados";
const NO_SHOWS: &str = "Todavía no hay programas guardados.";
const ADD_HEADING: &str = "Agregar un programa";
const SEARCH_PLACEHOLDER: &str = "Busca el programa por su nombre";
const SEARCH: &str = "Buscar";
const SEARCHING: &str = "Buscando en el directorio de Apple…";
const NO_RESULTS: &str = "No se encontró ningún programa con ese nombre.";
const SEARCH_FAILED: &str = "No se pudo buscar en el directorio de Apple. Revisa la conexión a internet.";
const PICK_ONE: &str = "Elige el programa que quieres agregar:";
const CONFIRM: &str = "Sí, agregar";
const CANCEL: &str = "Cancelar";
const FEED_PLACEHOLDER: &str = "O pega aquí la dirección del feed RSS";
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
    search_input: String,
    results: Vec<SavedShow>,
    pending: Option<SavedShow>,
    feed_input: String,
    busy: bool,
    notice: Option<&'static str>,
}

#[derive(Debug, Clone)]
enum Message {
    SearchInput(String),
    Search,
    Found(Result<Vec<SavedShow>, &'static str>),
    Pick(SavedShow),
    Confirm,
    Cancel,
    FeedInput(String),
    AddFeed,
    Fetched(Result<SavedShow, &'static str>),
    Select(String),
    Remove,
}

pub fn run() -> iced::Result {
    iced::application(boot, update, view).title(TITLE).window_size(Size::new(640.0, 600.0)).run()
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
    Window {
        list_path,
        shows,
        selected: None,
        search_input: String::new(),
        results: Vec::new(),
        pending: None,
        feed_input: String::new(),
        busy: false,
        notice,
    }
}

fn search(term: String) -> Result<Vec<SavedShow>, &'static str> {
    directory::search(&term).map_err(|error| {
        eprintln!("searching for {term}: {error:#}");
        SEARCH_FAILED
    })
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

fn is_saved(window: &Window, feed: &str) -> bool {
    window.shows.iter().any(|saved| saved.feed == feed)
}

fn update(window: &mut Window, message: Message) -> Task<Message> {
    match message {
        Message::SearchInput(input) => window.search_input = input,
        Message::Search => {
            let term = window.search_input.trim().to_string();
            if term.is_empty() || window.busy {
                return Task::none();
            }
            window.busy = true;
            window.results.clear();
            window.pending = None;
            window.notice = Some(SEARCHING);
            return Task::perform(async move { search(term) }, Message::Found);
        }
        Message::Found(result) => {
            window.busy = false;
            match result {
                Ok(results) => {
                    window.notice = if results.is_empty() { Some(NO_RESULTS) } else { None };
                    window.results = results;
                }
                Err(notice) => window.notice = Some(notice),
            }
        }
        Message::Pick(show) => {
            if is_saved(window, &show.feed) {
                window.notice = Some(ALREADY_SAVED);
                window.pending = None;
            } else {
                window.notice = None;
                window.pending = Some(show);
            }
        }
        Message::Cancel => window.pending = None,
        Message::Confirm => {
            let Some(show) = window.pending.clone() else { return Task::none() };
            if window.busy {
                return Task::none();
            }
            window.busy = true;
            window.notice = Some(FETCHING);
            return Task::perform(async move { fetch_show(show.feed.clone()).map(|_| show) }, Message::Fetched);
        }
        Message::FeedInput(input) => window.feed_input = input,
        Message::AddFeed => {
            let url = window.feed_input.trim().to_string();
            if url.is_empty() || window.busy {
                return Task::none();
            }
            if is_saved(window, &url) {
                window.notice = Some(ALREADY_SAVED);
                return Task::none();
            }
            window.busy = true;
            window.notice = Some(FETCHING);
            return Task::perform(async move { fetch_show(url) }, Message::Fetched);
        }
        Message::Fetched(result) => {
            window.busy = false;
            window.notice = None;
            match result {
                Ok(show) => {
                    if shows::add(&mut window.shows, show) {
                        window.feed_input.clear();
                        window.search_input.clear();
                        window.results.clear();
                        window.pending = None;
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

fn label(show: &SavedShow) -> String {
    if show.publisher.is_empty() { show.title.clone() } else { format!("{}  ·  {}", show.title, show.publisher) }
}

fn confirm_question(show: &SavedShow) -> String {
    if show.publisher.is_empty() {
        format!("¿Agregar «{}»?", show.title)
    } else {
        format!("¿Agregar «{}», de {}?", show.title, show.publisher)
    }
}

fn show_row<'a>(show: &SavedShow, selected: bool, message: Message) -> Element<'a, Message> {
    let style = if selected { button::primary } else { button::text };
    button(text(label(show))).width(Length::Fill).style(style).on_press(message).into()
}

fn view(window: &Window) -> Element<'_, Message> {
    let list: Element<'_, Message> = if window.shows.is_empty() {
        text(NO_SHOWS).into()
    } else {
        let rows = window.shows.iter().map(|show| {
            let selected = window.selected.as_deref() == Some(show.feed.as_str());
            show_row(show, selected, Message::Select(show.feed.clone()))
        });
        scrollable(column(rows).spacing(2)).into()
    };
    let remove = button(REMOVE).on_press_maybe(window.selected.as_ref().map(|_| Message::Remove));

    let search_input = text_input(SEARCH_PLACEHOLDER, &window.search_input).id("search").on_input(Message::SearchInput).on_submit(Message::Search);
    let search = button(SEARCH).on_press_maybe((!window.busy).then_some(Message::Search));
    let mut add = column![text(ADD_HEADING).size(22), row![search_input, search].spacing(8)].spacing(10);
    if let Some(show) = &window.pending {
        let confirm = button(CONFIRM).on_press_maybe((!window.busy).then_some(Message::Confirm));
        let cancel = button(CANCEL).style(button::secondary).on_press(Message::Cancel);
        add = add.push(row![text(confirm_question(show)).width(Length::Fill), confirm, cancel].spacing(8).align_y(Alignment::Center));
    } else if !window.results.is_empty() {
        let rows = window.results.iter().map(|show| show_row(show, false, Message::Pick(show.clone())));
        add = add.push(text(PICK_ONE)).push(container(column(rows).spacing(2)).padding(6).width(Length::Fill).style(container::bordered_box));
    }

    let feed_input = text_input(FEED_PLACEHOLDER, &window.feed_input).id("feed").on_input(Message::FeedInput).on_submit(Message::AddFeed);
    let add_feed = button(ADD_FEED).on_press_maybe((!window.busy).then_some(Message::AddFeed));
    column![
        text(SHOWS_HEADING).size(22),
        container(list).padding(6).width(Length::Fill).height(Length::Fill).style(container::bordered_box),
        row![remove].spacing(8),
        rule::horizontal(1),
        add,
        row![feed_input, add_feed].spacing(8),
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

    fn radio_duna() -> SavedShow {
        SavedShow { title: "Nada Personal".into(), publisher: "Radio Duna".into(), feed: FEED.into() }
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
        assert!(window.busy);
        assert_eq!(window.notice, Some(FETCHING));

        let _ = update(&mut window, Message::Fetched(Ok(radio_duna())));
        assert!(window.feed_input.is_empty());

        let mut window = open(path.clone());
        assert_eq!(window.shows, [radio_duna()]);
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
        let _ = update(&mut window, Message::Fetched(Ok(radio_duna())));
        let _ = update(&mut window, Message::FeedInput(FEED.into()));
        let _ = update(&mut window, Message::AddFeed);
        assert_eq!(window.notice, Some(ALREADY_SAVED));
        assert!(!window.busy);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn search_result_is_added_only_after_the_confirm_step() {
        let dir = std::env::temp_dir().join(format!("podcast-monitor-gui-search-{}", std::process::id()));
        let path = shows::list_path(&dir);
        let mut window = open(path.clone());
        drive(&mut window, |ui| {
            ui.click(iced_test::selector::id("search")).unwrap();
            ui.typewrite("nada personal");
            ui.click(SEARCH).unwrap();
        });
        assert_eq!(window.search_input, "nada personal");
        assert_eq!(window.notice, Some(SEARCHING));

        let other = SavedShow { title: "Nada Personal".into(), publisher: "Nada Personal".into(), feed: "https://b/feed".into() };
        let _ = update(&mut window, Message::Found(Ok(vec![other.clone(), radio_duna()])));
        drive(&mut window, |ui| {
            ui.click("Nada Personal  ·  Radio Duna").unwrap();
        });
        assert_eq!(window.pending, Some(radio_duna()));
        assert!(window.shows.is_empty());
        assert!(simulator(view(&window)).find("¿Agregar «Nada Personal», de Radio Duna?").is_ok());

        drive(&mut window, |ui| {
            ui.click(CANCEL).unwrap();
        });
        assert!(window.pending.is_none());
        drive(&mut window, |ui| {
            ui.click("Nada Personal  ·  Nada Personal").unwrap();
        });
        assert_eq!(window.pending, Some(other));
        let _ = update(&mut window, Message::Pick(radio_duna()));
        drive(&mut window, |ui| {
            ui.click(CONFIRM).unwrap();
        });
        assert!(window.busy);
        assert!(window.shows.is_empty());
        let _ = update(&mut window, Message::Fetched(Ok(radio_duna())));
        assert!(window.results.is_empty() && window.pending.is_none());
        assert_eq!(open(path).shows, [radio_duna()]);

        let _ = update(&mut window, Message::Found(Ok(vec![radio_duna()])));
        let _ = update(&mut window, Message::Pick(radio_duna()));
        assert_eq!(window.notice, Some(ALREADY_SAVED));
        assert!(window.pending.is_none());
        let _ = update(&mut window, Message::Found(Ok(Vec::new())));
        assert_eq!(window.notice, Some(NO_RESULTS));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    #[ignore = "writes headless renders of the window for a look at the layout"]
    fn writes_snapshots() {
        let dir = std::env::temp_dir().join("podcast-monitor-gui-snapshots");
        let _ = std::fs::remove_dir_all(&dir);
        let mut window = open(shows::list_path(&dir.join("data")));
        let animales = SavedShow { title: "Animales Políticos".into(), publisher: "Casa Editorial Lo Que Importa".into(), feed: "https://anchor.fm/s/106ac41b0/podcast/rss".into() };
        window.shows = vec![animales, radio_duna()];
        window.search_input = "nada personal".into();
        window.results = vec![
            radio_duna(),
            SavedShow { title: "Nada Personal - Podcast".into(), publisher: "Nada Personal".into(), feed: "https://b/feed".into() },
        ];
        let size = Size::new(640.0, 600.0);
        let theme = iced::Theme::Light;
        let snapshot = |window: &Window, name: &str| {
            let mut ui = iced_test::Simulator::with_size(Default::default(), size, view(window));
            ui.snapshot(&theme).unwrap().matches_image(dir.join(name)).unwrap();
        };
        snapshot(&window, "results.png");
        let _ = update(&mut window, Message::Pick(SavedShow { title: "Nada Personal - Podcast".into(), publisher: "Nada Personal".into(), feed: "https://b/feed".into() }));
        snapshot(&window, "confirm.png");
        println!("{}", dir.display());
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
