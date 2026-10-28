use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Local;
use iced::futures::channel::mpsc;
use iced::widget::{button, column, container, progress_bar, radio, row, rule, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Size, Task};

use crate::directory;
use crate::feed::{self, Episode};
use crate::job::{self, Failure, Request, Settings, Stage};
use crate::section::{spanish_date, spanish_duration};
use crate::shows::{self, SavedShow};
use crate::summarize::{self, Provider};

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
const EPISODES_HEADING: &str = "Episodios";
const PICK_SHOW: &str = "Elige un programa guardado para ver sus episodios.";
const LOADING_EPISODES: &str = "Cargando episodios…";
const EPISODES_FAILED: &str = "No se pudieron cargar los episodios. Revisa la conexión a internet.";
const ASSISTANT: &str = "Asistente para los resúmenes:";
const START: &str = "Generar informe";
const CONTINUE: &str = "Continuar";
const PREPARING: &str = "Descargando y preparando el audio…";
const REPORT: &str = "Armando el PDF…";
const FAILED: &str = "El informe no se pudo terminar. Lo ya terminado quedó guardado; puedes intentarlo de nuevo con Continuar.";
const OPEN_PDF: &str = "Abrir PDF";
const EPISODE_COUNT: usize = 10;

struct Window {
    data_dir: PathBuf,
    list_path: Option<PathBuf>,
    shows: Vec<SavedShow>,
    selected: Option<String>,
    search_input: String,
    results: Vec<SavedShow>,
    confirming: Option<SavedShow>,
    feed_input: String,
    busy: bool,
    notice: Option<&'static str>,
    provider: Provider,
    episodes: Vec<Episode>,
    episodes_loading: bool,
    episodes_failed: bool,
    picked: Option<String>,
    unfinished: Option<Request>,
    running: Option<(Request, Stage)>,
    status: Option<String>,
    report: Option<PathBuf>,
    opener: fn(&Path) -> std::io::Result<()>,
}

#[derive(Debug, Clone)]
enum RunEvent {
    Stage(Stage),
    Finished(Result<PathBuf, Failure>),
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
    Episodes(String, Result<Vec<Episode>, ()>),
    PickEpisode(String),
    Provider(Provider),
    Start,
    Continue,
    Run(RunEvent),
    OpenPdf,
}

pub fn run() -> iced::Result {
    iced::application(boot, update, view).title(TITLE).window_size(Size::new(1000.0, 640.0)).run()
}

fn boot() -> Window {
    open(shows::data_dir())
}

fn open(data_dir: PathBuf) -> Window {
    let path = shows::list_path(&data_dir);
    let (list_path, shows, notice) = match shows::load(&path) {
        Ok(shows) => (Some(path), shows, None),
        Err(error) => {
            log::error!("loading the show list: {error:#}");
            (None, Vec::new(), Some(LOAD_FAILED))
        }
    };
    let unfinished = job::unfinished(&data_dir).unwrap_or_else(|error| {
        log::error!("reading the unfinished run: {error:#}");
        None
    });
    let provider = job::settings(&data_dir).provider;
    Window {
        data_dir,
        list_path,
        shows,
        selected: None,
        search_input: String::new(),
        results: Vec::new(),
        confirming: None,
        feed_input: String::new(),
        busy: false,
        notice,
        provider,
        episodes: Vec::new(),
        episodes_loading: false,
        episodes_failed: false,
        picked: None,
        unfinished,
        running: None,
        status: None,
        report: None,
        opener: open_with_system,
    }
}

fn open_with_system(path: &Path) -> std::io::Result<()> {
    open::that(path)
}

fn search(term: String) -> Result<Vec<SavedShow>, &'static str> {
    directory::search(&term).map_err(|error| {
        log::warn!("searching for {term}: {error:#}");
        SEARCH_FAILED
    })
}

fn fetch_show(url: String) -> Result<SavedShow, &'static str> {
    let fetched = feed::fetch(&url).map_err(|error| {
        log::warn!("adding {url}: {error:#}");
        BAD_FEED
    })?;
    if fetched.episodes.is_empty() {
        return Err(NO_AUDIO);
    }
    Ok(SavedShow { title: fetched.title, publisher: fetched.author, feed: url })
}

fn newest_episodes(url: String) -> Result<Vec<Episode>, ()> {
    feed::fetch(&url).map(|fetched| feed::newest(fetched.episodes, EPISODE_COUNT)).map_err(|error| log::warn!("listing episodes of {url}: {error:#}"))
}

fn save(window: &mut Window) {
    let Some(path) = &window.list_path else { return };
    if let Err(error) = shows::save(path, &window.shows) {
        log::error!("saving the show list: {error:#}");
        window.notice = Some(SAVE_FAILED);
    }
}

fn is_saved(window: &Window, feed: &str) -> bool {
    window.shows.iter().any(|saved| saved.feed == feed)
}

fn provider_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Claude => "Claude",
        Provider::Codex => "Codex",
    }
}

fn program_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Claude => "claude",
        Provider::Codex => "codex",
    }
}

fn stage_text(stage: Stage, provider: Provider) -> String {
    match stage {
        Stage::Preparing => PREPARING.into(),
        Stage::Transcribing(percent) => format!("Transcribiendo: {percent} %"),
        Stage::Summarizing => format!("Resumiendo con {}…", provider_name(provider)),
        Stage::Report => REPORT.into(),
    }
}

fn failure_text(failure: &Failure, provider: Provider) -> String {
    match failure {
        Failure::MissingModel(path) => format!(
            "Falta el modelo de transcripción. Copia {} en la carpeta {} y vuelve a intentarlo con Continuar.",
            job::MODEL_FILE,
            path.parent().map(|dir| dir.display().to_string()).unwrap_or_default()
        ),
        Failure::Limit => format!(
            "{} llegó a su límite de uso. Lo ya terminado quedó guardado; vuelve a intentarlo más tarde con Continuar.",
            provider_name(provider)
        ),
        Failure::MissingCli => format!(
            "No se encontró el programa {}. Instálalo e inicia sesión una vez en la Terminal; luego usa Continuar.",
            program_name(provider)
        ),
        Failure::Other => FAILED.into(),
    }
}

fn begin(window: &mut Window, request: Request) -> Task<Message> {
    if window.running.is_some() {
        return Task::none();
    }
    if let Err(error) = job::mark_unfinished(&window.data_dir, &request) {
        log::error!("saving the unfinished run: {error:#}");
    }
    window.unfinished = Some(request.clone());
    window.running = Some((request.clone(), Stage::Preparing));
    window.status = None;
    window.report = None;
    let (sender, receiver) = mpsc::unbounded();
    let data_dir = window.data_dir.clone();
    let provider = window.provider;
    std::thread::spawn(move || {
        let stages = sender.clone();
        let sink: job::StageSink = Arc::new(move |stage| {
            let _ = stages.unbounded_send(RunEvent::Stage(stage));
        });
        let outcome = job::run(&data_dir, &request, provider, &summarize::Cli { provider }, sink);
        let _ = sender.unbounded_send(RunEvent::Finished(outcome));
    });
    Task::run(receiver, Message::Run)
}

fn update(window: &mut Window, message: Message) -> Task<Message> {
    if !matches!(message, Message::SearchInput(_) | Message::FeedInput(_) | Message::Run(RunEvent::Stage(Stage::Transcribing(_)))) {
        log::debug!("window: {message:?}");
    }
    match message {
        Message::SearchInput(input) => window.search_input = input,
        Message::Search => {
            let term = window.search_input.trim().to_string();
            if term.is_empty() || window.busy {
                return Task::none();
            }
            window.busy = true;
            window.results.clear();
            window.confirming = None;
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
                window.confirming = None;
            } else {
                window.notice = None;
                window.confirming = Some(show);
            }
        }
        Message::Cancel => window.confirming = None,
        Message::Confirm => {
            let Some(show) = window.confirming.clone() else { return Task::none() };
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
                        window.confirming = None;
                        save(window);
                    } else {
                        window.notice = Some(ALREADY_SAVED);
                    }
                }
                Err(notice) => window.notice = Some(notice),
            }
        }
        Message::Select(feed) => {
            if window.selected.as_deref() == Some(feed.as_str()) {
                return Task::none();
            }
            window.selected = Some(feed.clone());
            window.episodes.clear();
            window.picked = None;
            window.episodes_loading = true;
            window.episodes_failed = false;
            let url = feed.clone();
            return Task::perform(async move { newest_episodes(url) }, move |result| Message::Episodes(feed.clone(), result));
        }
        Message::Remove => {
            if let Some(feed) = window.selected.take() {
                shows::remove(&mut window.shows, &feed);
                window.episodes.clear();
                window.picked = None;
                window.episodes_loading = false;
                save(window);
            }
        }
        Message::Episodes(feed, result) => {
            if window.selected.as_deref() != Some(feed.as_str()) {
                return Task::none();
            }
            window.episodes_loading = false;
            match result {
                Ok(episodes) => window.episodes = episodes,
                Err(()) => window.episodes_failed = true,
            }
        }
        Message::PickEpisode(id) => window.picked = Some(id),
        Message::Provider(provider) => {
            window.provider = provider;
            log::info!("provider set to {provider:?}");
            if let Err(error) = job::save_settings(&window.data_dir, &Settings { provider, ..job::settings(&window.data_dir) }) {
                log::error!("saving settings: {error:#}");
            }
        }
        Message::Start => {
            let show = window.shows.iter().find(|show| window.selected.as_deref() == Some(show.feed.as_str())).cloned();
            let episode = window.episodes.iter().find(|episode| window.picked.as_deref() == Some(episode.id().as_str())).cloned();
            if let (Some(show), Some(episode)) = (show, episode) {
                return begin(window, Request { show, episode });
            }
        }
        Message::Continue => {
            if let Some(request) = window.unfinished.clone() {
                return begin(window, request);
            }
        }
        Message::Run(RunEvent::Stage(stage)) => {
            if let Some((_, current)) = window.running.as_mut() {
                *current = stage;
            }
        }
        Message::Run(RunEvent::Finished(outcome)) => {
            let Some((request, _)) = window.running.take() else { return Task::none() };
            match outcome {
                Ok(pdf) => {
                    log::info!("report written to {}", pdf.display());
                    if let Err(error) = job::clear_unfinished(&window.data_dir) {
                        log::error!("clearing the unfinished run: {error:#}");
                    }
                    window.unfinished = None;
                    window.status = Some(format!("Informe listo: «{}».", request.episode.title));
                    window.report = Some(pdf);
                }
                Err(failure) => window.status = Some(failure_text(&failure, window.provider)),
            }
        }
        Message::OpenPdf => {
            if let Some(pdf) = &window.report
                && let Err(error) = (window.opener)(pdf)
            {
                log::error!("opening {}: {error}", pdf.display());
                window.status = Some(format!("No se pudo abrir el PDF. Está en {}", pdf.display()));
            }
        }
    }
    Task::none()
}

fn label(show: &SavedShow) -> String {
    if show.publisher.is_empty() { show.title.clone() } else { format!("{}  ·  {}", show.title, show.publisher) }
}

fn episode_label(episode: &Episode) -> String {
    let mut parts = Vec::new();
    if let Some(published) = episode.published {
        parts.push(spanish_date(published.with_timezone(&Local).date_naive()));
    }
    parts.push(episode.title.clone());
    if episode.duration_s > 0 {
        parts.push(spanish_duration(episode.duration_s));
    }
    parts.join("  ·  ")
}

fn confirm_question(show: &SavedShow) -> String {
    if show.publisher.is_empty() {
        format!("¿Agregar «{}»?", show.title)
    } else {
        format!("¿Agregar «{}», de {}?", show.title, show.publisher)
    }
}

fn selectable<'a>(label: String, selected: bool, message: Message) -> Element<'a, Message> {
    let style = if selected { button::primary } else { button::text };
    button(text(label)).width(Length::Fill).style(style).on_press(message).into()
}

fn boxed<'a>(content: impl Into<Element<'a, Message>>) -> container::Container<'a, Message> {
    container(content).padding(6).width(Length::Fill).style(container::bordered_box)
}

fn shows_panel(window: &Window) -> Element<'_, Message> {
    let list: Element<'_, Message> = if window.shows.is_empty() {
        text(NO_SHOWS).into()
    } else {
        let rows = window.shows.iter().map(|show| {
            let selected = window.selected.as_deref() == Some(show.feed.as_str());
            selectable(label(show), selected, Message::Select(show.feed.clone()))
        });
        scrollable(column(rows).spacing(2)).into()
    };
    let remove = button(REMOVE).on_press_maybe(window.selected.as_ref().map(|_| Message::Remove));

    let search_input = text_input(SEARCH_PLACEHOLDER, &window.search_input).id("search").on_input(Message::SearchInput).on_submit(Message::Search);
    let search = button(SEARCH).on_press_maybe((!window.busy).then_some(Message::Search));
    let mut add = column![text(ADD_HEADING).size(22), row![search_input, search].spacing(8)].spacing(10);
    if let Some(show) = &window.confirming {
        let confirm = button(CONFIRM).on_press_maybe((!window.busy).then_some(Message::Confirm));
        let cancel = button(CANCEL).style(button::secondary).on_press(Message::Cancel);
        add = add.push(row![text(confirm_question(show)).width(Length::Fill), confirm, cancel].spacing(8).align_y(Alignment::Center));
    } else if !window.results.is_empty() {
        let rows = window.results.iter().map(|show| selectable(label(show), false, Message::Pick(show.clone())));
        add = add.push(text(PICK_ONE)).push(boxed(column(rows).spacing(2)));
    }

    let feed_input = text_input(FEED_PLACEHOLDER, &window.feed_input).id("feed").on_input(Message::FeedInput).on_submit(Message::AddFeed);
    let add_feed = button(ADD_FEED).on_press_maybe((!window.busy).then_some(Message::AddFeed));
    column![
        text(SHOWS_HEADING).size(22),
        boxed(list).height(Length::Fill),
        row![remove].spacing(8),
        rule::horizontal(1),
        add,
        row![feed_input, add_feed].spacing(8),
        text(window.notice.unwrap_or("")),
    ]
    .spacing(10)
    .into()
}

fn run_panel(window: &Window) -> Element<'_, Message> {
    let list: Element<'_, Message> = if window.selected.is_none() {
        text(PICK_SHOW).into()
    } else if window.episodes_loading {
        text(LOADING_EPISODES).into()
    } else if window.episodes_failed {
        text(EPISODES_FAILED).into()
    } else {
        let rows = window.episodes.iter().map(|episode| {
            let id = episode.id();
            let picked = window.picked.as_deref() == Some(id.as_str());
            selectable(episode_label(episode), picked, Message::PickEpisode(id))
        });
        scrollable(column(rows).spacing(2)).into()
    };
    let assistant = row![
        text(ASSISTANT),
        radio(provider_name(Provider::Claude), Provider::Claude, Some(window.provider), Message::Provider),
        radio(provider_name(Provider::Codex), Provider::Codex, Some(window.provider), Message::Provider),
    ]
    .spacing(16)
    .align_y(Alignment::Center);
    let idle = window.running.is_none();
    let start = button(START).on_press_maybe((idle && window.picked.is_some()).then_some(Message::Start));

    let mut panel = column![text(EPISODES_HEADING).size(22), boxed(list).height(Length::Fill), assistant, start].spacing(10);
    match (&window.running, &window.unfinished) {
        (Some((request, stage)), _) => {
            panel = panel.push(text(format!("«{}»", request.episode.title))).push(text(stage_text(*stage, window.provider)));
            if let Stage::Transcribing(percent) = stage {
                panel = panel.push(progress_bar(0.0..=100.0, *percent as f32));
            }
        }
        (None, Some(request)) => {
            let question = text(format!("Quedó un informe sin terminar: «{}».", request.episode.title)).width(Length::Fill);
            panel = panel.push(row![question, button(CONTINUE).on_press(Message::Continue)].spacing(8).align_y(Alignment::Center));
        }
        (None, None) => {}
    }
    if let Some(status) = &window.status {
        panel = panel.push(text(status.clone()));
    }
    if window.report.is_some() {
        panel = panel.push(button(OPEN_PDF).on_press(Message::OpenPdf));
    }
    panel.into()
}

fn view(window: &Window) -> Element<'_, Message> {
    row![
        container(shows_panel(window)).width(Length::FillPortion(1)),
        container(run_panel(window)).width(Length::FillPortion(1)),
    ]
    .spacing(24)
    .padding(12)
    .into()
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

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

    fn data_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("podcast-monitor-gui-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn radio_duna() -> SavedShow {
        SavedShow { title: "Nada Personal".into(), publisher: "Radio Duna".into(), feed: FEED.into() }
    }

    fn episode() -> Episode {
        Episode {
            guid: "np-1".into(),
            title: "Episodio de prueba".into(),
            published: DateTime::parse_from_rfc3339("2026-10-26T15:00:00-03:00").ok(),
            duration_s: 3559,
            audio_url: "https://example.com/np-1.mp3".into(),
            description: String::new(),
        }
    }

    #[test]
    fn add_by_feed_url_then_remove_survives_a_reopen() {
        let dir = data_dir("add");
        let mut window = open(dir.clone());
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

        let mut window = open(dir.clone());
        assert_eq!(window.shows, [radio_duna()]);
        drive(&mut window, |ui| {
            ui.click("Nada Personal  ·  Radio Duna").unwrap();
        });
        assert!(window.episodes_loading);
        drive(&mut window, |ui| {
            ui.click(REMOVE).unwrap();
        });
        assert!(open(dir.clone()).shows.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_failed_fetch_or_a_saved_feed_shows_a_notice() {
        let dir = data_dir("notice");
        let mut window = open(dir.clone());
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
        let dir = data_dir("search");
        let mut window = open(dir.clone());
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
        assert_eq!(window.confirming, Some(radio_duna()));
        assert!(window.shows.is_empty());
        assert!(simulator(view(&window)).find("¿Agregar «Nada Personal», de Radio Duna?").is_ok());

        drive(&mut window, |ui| {
            ui.click(CANCEL).unwrap();
        });
        assert!(window.confirming.is_none());
        drive(&mut window, |ui| {
            ui.click("Nada Personal  ·  Nada Personal").unwrap();
        });
        assert_eq!(window.confirming, Some(other));
        let _ = update(&mut window, Message::Pick(radio_duna()));
        drive(&mut window, |ui| {
            ui.click(CONFIRM).unwrap();
        });
        assert!(window.busy);
        assert!(window.shows.is_empty());
        let _ = update(&mut window, Message::Fetched(Ok(radio_duna())));
        assert!(window.results.is_empty() && window.confirming.is_none());
        assert_eq!(open(dir.clone()).shows, [radio_duna()]);

        let _ = update(&mut window, Message::Found(Ok(vec![radio_duna()])));
        let _ = update(&mut window, Message::Pick(radio_duna()));
        assert_eq!(window.notice, Some(ALREADY_SAVED));
        assert!(window.confirming.is_none());
        let _ = update(&mut window, Message::Found(Ok(Vec::new())));
        assert_eq!(window.notice, Some(NO_RESULTS));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn picked_episode_runs_stops_at_a_limit_and_continues_after_a_reopen() {
        let dir = data_dir("run");
        let mut window = open(dir.clone());
        let _ = update(&mut window, Message::Fetched(Ok(radio_duna())));
        let _ = update(&mut window, Message::Select(FEED.into()));
        let _ = update(&mut window, Message::Episodes(FEED.into(), Ok(vec![episode()])));
        let row = "26 de octubre de 2026  ·  Episodio de prueba  ·  59 min";
        let _ = update(&mut window, Message::Provider(Provider::Codex));
        drive(&mut window, |ui| {
            ui.click(row).unwrap();
        });
        assert_eq!(window.provider, Provider::Codex);
        assert_eq!(window.picked, Some(episode().id()));
        drive(&mut window, |ui| {
            ui.click(START).unwrap();
        });
        let request = Request { show: radio_duna(), episode: episode() };
        assert_eq!(window.running, Some((request.clone(), Stage::Preparing)));
        assert_eq!(job::unfinished(&dir).unwrap(), Some(request.clone()));

        let _ = update(&mut window, Message::Run(RunEvent::Stage(Stage::Transcribing(40))));
        assert!(simulator(view(&window)).find("Transcribiendo: 40 %").is_ok());
        let _ = update(&mut window, Message::Run(RunEvent::Stage(Stage::Summarizing)));
        assert!(simulator(view(&window)).find("Resumiendo con Codex…").is_ok());
        let _ = update(&mut window, Message::Run(RunEvent::Finished(Err(Failure::Limit))));
        assert!(window.running.is_none());
        assert!(simulator(view(&window)).find(failure_text(&Failure::Limit, Provider::Codex).as_str()).is_ok());

        let mut window = open(dir.clone());
        assert_eq!(window.provider, Provider::Codex);
        assert!(simulator(view(&window)).find("Quedó un informe sin terminar: «Episodio de prueba».").is_ok());
        drive(&mut window, |ui| {
            ui.click(CONTINUE).unwrap();
        });
        assert_eq!(window.running, Some((request, Stage::Preparing)));
        let _ = update(&mut window, Message::Run(RunEvent::Finished(Ok(dir.join("informe.pdf")))));
        assert!(window.unfinished.is_none());
        assert_eq!(job::unfinished(&dir).unwrap(), None);
        assert!(simulator(view(&window)).find("Informe listo: «Episodio de prueba».").is_ok());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    thread_local! {
        static OPENED: std::cell::RefCell<Vec<PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    fn record(path: &Path) -> std::io::Result<()> {
        OPENED.with(|opened| opened.borrow_mut().push(path.to_path_buf()));
        Ok(())
    }

    fn refuse(_path: &Path) -> std::io::Result<()> {
        Err(std::io::Error::other("no viewer"))
    }

    #[test]
    fn open_pdf_appears_after_a_finished_run_and_opens_that_file() {
        let dir = data_dir("open");
        let mut window = open(dir.clone());
        window.opener = record;
        assert!(simulator(view(&window)).find(OPEN_PDF).is_err());
        window.running = Some((Request { show: radio_duna(), episode: episode() }, Stage::Report));
        let pdf = dir.join("runs").join("nada-personal").join(episode().id()).join(job::REPORT_FILE);
        let _ = update(&mut window, Message::Run(RunEvent::Finished(Ok(pdf.clone()))));
        drive(&mut window, |ui| {
            ui.click(OPEN_PDF).unwrap();
        });
        assert_eq!(OPENED.with(|opened| opened.borrow().clone()), [pdf.clone()]);

        window.opener = refuse;
        let _ = update(&mut window, Message::OpenPdf);
        assert_eq!(window.status, Some(format!("No se pudo abrir el PDF. Está en {}", pdf.display())));
    }

    #[test]
    fn failures_read_in_spanish() {
        let missing = failure_text(&Failure::MissingModel(PathBuf::from("datos").join("models").join(job::MODEL_FILE)), Provider::Claude);
        assert!(missing.starts_with("Falta el modelo de transcripción. Copia ggml-large-v3-turbo.bin en la carpeta datos"));
        assert!(failure_text(&Failure::MissingCli, Provider::Claude).contains("programa claude."));
        assert!(failure_text(&Failure::Limit, Provider::Claude).starts_with("Claude llegó a su límite de uso."));
    }

    #[test]
    #[ignore = "writes headless renders of the window for a look at the layout"]
    fn writes_snapshots() {
        let dir = std::env::temp_dir().join("podcast-monitor-gui-snapshots");
        let _ = std::fs::remove_dir_all(&dir);
        let mut window = open(dir.join("data"));
        let animales = SavedShow { title: "Animales Políticos".into(), publisher: "Casa Editorial Lo Que Importa".into(), feed: "https://anchor.fm/s/106ac41b0/podcast/rss".into() };
        window.shows = vec![animales, radio_duna()];
        window.search_input = "nada personal".into();
        window.results = vec![
            radio_duna(),
            SavedShow { title: "Nada Personal - Podcast".into(), publisher: "Nada Personal".into(), feed: "https://b/feed".into() },
        ];
        window.selected = Some(FEED.into());
        window.episodes = vec![episode()];
        window.picked = Some(episode().id());
        window.running = Some((Request { show: radio_duna(), episode: episode() }, Stage::Transcribing(45)));
        let size = Size::new(1000.0, 640.0);
        let theme = iced::Theme::Light;
        let snapshot = |window: &Window, name: &str| {
            let mut ui = iced_test::Simulator::with_size(Default::default(), size, view(window));
            ui.snapshot(&theme).unwrap().matches_image(dir.join(name)).unwrap();
        };
        snapshot(&window, "running.png");
        let _ = update(&mut window, Message::Pick(SavedShow { title: "Nada Personal - Podcast".into(), publisher: "Nada Personal".into(), feed: "https://b/feed".into() }));
        window.running = None;
        window.unfinished = Some(Request { show: radio_duna(), episode: episode() });
        window.status = Some(failure_text(&Failure::Limit, Provider::Claude));
        snapshot(&window, "limit.png");
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
        assert_eq!(newest_episodes(FEED.into()).unwrap().len(), EPISODE_COUNT);
    }
}
