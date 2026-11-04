use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Local;
use iced::futures::channel::mpsc;
use iced::widget::{button, column, container, progress_bar, radio, row, rule, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Size, Task};

use crate::assistant;
use crate::directory;
use crate::feed::{self, Episode};
use crate::job::{self, Failure, Request, Settings, Stage};
use crate::model;
use crate::section::{spanish_date, spanish_duration};
use crate::shows::{self, SavedShow};
use crate::summarize::{self, Provider};
use crate::week::{self, Span, Week};
use crate::weekly;

const TITLE: &str = concat!("Podcast-Monitor ", env!("CARGO_PKG_VERSION"));
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
const RESUME_NOW: &str = "Reanudar ahora";
const LOW_MEMORY: &str = "El equipo estuvo sin memoria libre durante 15 minutos. Lo ya terminado quedó guardado; cierra otros programas y usa Continuar.";
const START_WEEK: &str = "Generar informe semanal";
const CONTINUE_WEEK: &str = "Continuar informe semanal";
const COLLECTING: &str = "Buscando los episodios de esta semana…";
const WEEK_SUMMARIES: &str = "Resumiendo la semana de cada programa y armando el PDF…";
const MODEL_NEEDED: &str = "Para transcribir, la app necesita el modelo de voz (1,6 GB). Se descarga una sola vez.";
const DOWNLOAD_MODEL: &str = "Descargar modelo";
const MODEL_CHECKING: &str = "Revisando que el modelo esté completo…";
const MODEL_READY: &str = "Modelo de voz listo.";
const MODEL_INTERRUPTED: &str = "La descarga se interrumpió. Revisa la conexión a internet y usa Descargar modelo para seguir donde quedó.";
const MODEL_DAMAGED: &str = "El modelo descargado llegó dañado y se borró. Usa Descargar modelo para bajarlo de nuevo.";
const MISSING_MODEL: &str = "Falta el modelo de voz. Usa Descargar modelo y luego Continuar.";
const SET_UP_ASSISTANT: &str = "Instalar y conectar";
const SIGN_IN: &str = "Iniciar sesión";
const SIGN_IN_NOW: &str = "Inicia sesión en la ventana del navegador que se abrió.";
const SIGN_IN_FAILED: &str = "No se completó el inicio de sesión. Vuelve a intentarlo con Iniciar sesión.";
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
    unfinished_week: bool,
    week: Option<WeekState>,
    model_ready: bool,
    model_download: Option<model::Progress>,
    model_notice: Option<&'static str>,
    assistant: Option<assistant::Status>,
    assistant_step: Option<assistant::Step>,
    assistant_notice: Option<String>,
    assistant_cancel: Arc<AtomicBool>,
    opener: fn(&Path) -> std::io::Result<()>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct WeekState {
    position: Option<(usize, usize, String)>,
    stage: Option<Stage>,
    building: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct WeekDone {
    pdf: PathBuf,
    shows: usize,
    left_out: usize,
}

#[derive(Debug, Clone)]
enum WeekEvent {
    Episode(usize, usize, String),
    Stage(Stage),
    Building,
    Finished(Result<WeekDone, Failure>),
}

#[derive(Debug, Clone)]
enum ModelEvent {
    Progress(model::Progress),
    Finished(Result<(), model::Failure>),
}

#[derive(Debug, Clone)]
enum AssistantEvent {
    Step(assistant::Step),
    Finished(assistant::Status, Result<(), assistant::Failure>),
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
    StartWeek,
    Week(WeekEvent),
    ResumeNow,
    OpenPdf,
    DownloadModel,
    Model(ModelEvent),
    AssistantChecked(Provider, assistant::Status),
    SetUpAssistant,
    CancelAssistant,
    Assistant(AssistantEvent),
}

const ICON_PNG: &[u8] = include_bytes!("../assets/brand/windows/podcast-monitor-icon-256.png");

pub fn run() -> iced::Result {
    let settings = iced::window::Settings { size: Size::new(1000.0, 640.0), icon: window_icon(), ..Default::default() };
    iced::application(boot, update, view).title(TITLE).window(settings).run()
}

fn window_icon() -> Option<iced::window::Icon> {
    match decode_rgba(ICON_PNG) {
        Ok((rgba, width, height)) => iced::window::icon::from_rgba(rgba, width, height).ok(),
        Err(error) => {
            log::warn!("window icon not decoded: {error}");
            None
        }
    }
}

fn decode_rgba(png_bytes: &[u8]) -> anyhow::Result<(Vec<u8>, u32, u32)> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
    decoder.set_transformations(png::Transformations::ALPHA | png::Transformations::EXPAND);
    let mut reader = decoder.read_info()?;
    let mut rgba = vec![0; reader.output_buffer_size().ok_or_else(|| anyhow::anyhow!("icon too large"))?];
    let frame = reader.next_frame(&mut rgba)?;
    anyhow::ensure!(frame.color_type == png::ColorType::Rgba && frame.bit_depth == png::BitDepth::Eight, "icon is not 8-bit RGBA");
    rgba.truncate(frame.buffer_size());
    Ok((rgba, frame.width, frame.height))
}

fn boot() -> (Window, Task<Message>) {
    let window = open(shows::data_dir());
    let check = check_assistant(window.provider);
    (window, check)
}

fn check_assistant(provider: Provider) -> Task<Message> {
    Task::perform(async move { assistant::status(provider) }, move |status| Message::AssistantChecked(provider, status))
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
    let unfinished_week = has_unfinished_week(&data_dir);
    let model_ready = model::present(&data_dir);
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
        unfinished_week,
        week: None,
        model_ready,
        model_download: None,
        model_notice: None,
        assistant: None,
        assistant_step: None,
        assistant_notice: None,
        assistant_cancel: Arc::new(AtomicBool::new(false)),
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

fn gigabytes(megabytes: u64) -> String {
    format!("{:.1}", megabytes as f64 / 1024.0).replace('.', ",")
}

fn waiting_for_memory(window: &Window) -> bool {
    let single = window.running.as_ref().is_some_and(|(_, stage)| matches!(stage, Stage::WaitingForMemory(_)));
    let week = window.week.as_ref().is_some_and(|state| matches!(state.stage, Some(Stage::WaitingForMemory(_))));
    single || week
}

fn stage_text(stage: Stage, provider: Provider) -> String {
    match stage {
        Stage::Preparing => PREPARING.into(),
        Stage::Transcribing(percent) => format!("Transcribiendo: {percent} %"),
        Stage::WaitingForMemory(free) => format!(
            "Esperando memoria libre: hay {} GB libres y sigue sola al llegar a {} GB. Cierra otros programas o usa Reanudar ahora.",
            gigabytes(free),
            gigabytes(crate::memory::RESUME_MB)
        ),
        Stage::Summarizing => format!("Resumiendo con {}…", provider_name(provider)),
        Stage::Report => REPORT.into(),
    }
}

fn assistant_needed_text(provider: Provider) -> &'static str {
    match provider {
        Provider::Claude => "Para los resúmenes, la app necesita Claude. Se instala una sola vez y después inicias sesión con tu cuenta en el navegador. La app de escritorio de Claude no sirve para esto, pero la cuenta es la misma.",
        Provider::Codex => "Para los resúmenes, la app necesita Codex, de ChatGPT. Se instala una sola vez y después inicias sesión con tu cuenta de ChatGPT en el navegador. La app de escritorio de ChatGPT no sirve para esto, pero la cuenta es la misma.",
    }
}

fn assistant_failure_text(failure: assistant::Failure, provider: Provider) -> Option<String> {
    match failure {
        assistant::Failure::Install => Some(format!("No se pudo instalar {}. Revisa la conexión a internet y vuelve a intentarlo.", provider_name(provider))),
        assistant::Failure::SignIn => Some(SIGN_IN_FAILED.into()),
        assistant::Failure::Cancelled => None,
    }
}

fn failure_text(failure: &Failure, provider: Provider) -> String {
    match failure {
        Failure::MissingModel(_) => MISSING_MODEL.into(),
        Failure::Limit => format!(
            "{} llegó a su límite de uso. Lo ya terminado quedó guardado; vuelve a intentarlo más tarde con Continuar.",
            provider_name(provider)
        ),
        Failure::MissingCli => format!(
            "No se encontró {}. Usa Instalar y conectar y luego Continuar.",
            provider_name(provider)
        ),
        Failure::LowMemory => LOW_MEMORY.into(),
        Failure::Other => FAILED.into(),
    }
}

fn begin(window: &mut Window, request: Request) -> Task<Message> {
    if !idle(window) {
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
        let outcome = job::run(&data_dir, &request, provider, &summarize::Cli::new(provider), sink);
        let _ = sender.unbounded_send(RunEvent::Finished(outcome));
    });
    Task::run(receiver, Message::Run)
}

fn begin_assistant_setup(window: &mut Window) -> Task<Message> {
    if window.assistant_step.is_some() {
        return Task::none();
    }
    let provider = window.provider;
    window.assistant_notice = None;
    window.assistant_step = Some(if window.assistant == Some(assistant::Status::Missing) { assistant::Step::Installing } else { assistant::Step::SigningIn });
    window.assistant_cancel = Arc::new(AtomicBool::new(false));
    let cancel = window.assistant_cancel.clone();
    let (sender, receiver) = mpsc::unbounded();
    std::thread::spawn(move || {
        let steps = sender.clone();
        let outcome = assistant::set_up(provider, &cancel, &|step| {
            let _ = steps.unbounded_send(AssistantEvent::Step(step));
        });
        let _ = sender.unbounded_send(AssistantEvent::Finished(assistant::status(provider), outcome));
    });
    Task::run(receiver, Message::Assistant)
}

fn assistant_rows(window: &Window) -> Vec<Element<'_, Message>> {
    let mut rows: Vec<Element<'_, Message>> = Vec::new();
    let name = provider_name(window.provider);
    match (window.assistant_step, window.assistant) {
        (Some(step), _) => {
            let line = match step {
                assistant::Step::Installing => format!("Instalando {name}…"),
                assistant::Step::SigningIn => SIGN_IN_NOW.to_string(),
            };
            rows.push(text(line).into());
            rows.push(button(CANCEL).style(button::secondary).on_press(Message::CancelAssistant).into());
        }
        (None, Some(assistant::Status::Missing)) => {
            rows.push(text(assistant_needed_text(window.provider)).into());
            rows.push(button(SET_UP_ASSISTANT).on_press(Message::SetUpAssistant).into());
        }
        (None, Some(assistant::Status::SignedOut)) => {
            rows.push(text(format!("Falta iniciar sesión en {name} con tu cuenta. Se hace una sola vez, en el navegador.")).into());
            rows.push(button(SIGN_IN).on_press(Message::SetUpAssistant).into());
        }
        (None, _) => {}
    }
    if let Some(notice) = &window.assistant_notice {
        rows.push(text(notice.clone()).into());
    }
    rows
}

fn begin_model_download(window: &mut Window) -> Task<Message> {
    if window.model_ready || window.model_download.is_some() {
        return Task::none();
    }
    window.model_download = Some(model::Progress::Bytes(0, model::MODEL_SIZE));
    window.model_notice = None;
    let (sender, receiver) = mpsc::unbounded();
    let target = job::model_path(&window.data_dir);
    std::thread::spawn(move || {
        let progress = sender.clone();
        let outcome = model::fetch(&target, &model::HUGGING_FACE, &|step| {
            let _ = progress.unbounded_send(ModelEvent::Progress(step));
        });
        let _ = sender.unbounded_send(ModelEvent::Finished(outcome));
    });
    Task::run(receiver, Message::Model)
}

fn megabytes(bytes: u64) -> String {
    let digits = (bytes / 1_048_576).to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push('.');
        }
        grouped.push(digit);
    }
    grouped
}

fn model_progress_text(progress: model::Progress) -> String {
    match progress {
        model::Progress::Bytes(done, total) => format!("Descargando el modelo de voz: {} de {} MB ({} %)", megabytes(done), megabytes(total), done * 100 / total.max(1)),
        model::Progress::Checking => MODEL_CHECKING.into(),
    }
}

fn model_rows(window: &Window) -> Vec<Element<'_, Message>> {
    let mut rows: Vec<Element<'_, Message>> = Vec::new();
    match window.model_download {
        Some(progress) => {
            rows.push(text(model_progress_text(progress)).into());
            if let model::Progress::Bytes(done, total) = progress {
                rows.push(progress_bar(0.0..=100.0, (done as f64 * 100.0 / total.max(1) as f64) as f32).into());
            }
        }
        None if !window.model_ready => {
            rows.push(text(MODEL_NEEDED).into());
            rows.push(button(DOWNLOAD_MODEL).on_press(Message::DownloadModel).into());
        }
        None => {}
    }
    if let Some(notice) = window.model_notice {
        rows.push(text(notice).into());
    }
    rows
}

fn has_unfinished_week(data: &Path) -> bool {
    job::unfinished_week(data).unwrap_or_else(|error| {
        log::error!("reading the unfinished week: {error:#}");
        None
    }).is_some()
}

fn idle(window: &Window) -> bool {
    window.running.is_none() && window.week.is_none()
}

fn week_to_run(data: &Path, saved: &[SavedShow]) -> Week {
    match job::unfinished_week(data) {
        Ok(Some(stopped)) => Week { span: stopped.span, requests: stopped.requests, failed: Vec::new() },
        Ok(None) => week::collect(saved, Span::last_seven_days(&Local::now())),
        Err(error) => {
            log::error!("reading the unfinished week: {error:#}");
            week::collect(saved, Span::last_seven_days(&Local::now()))
        }
    }
}

fn week_job(data: &Path, saved: &[SavedShow], provider: Provider, sender: &mpsc::UnboundedSender<WeekEvent>) -> Result<WeekDone, Failure> {
    let week = week_to_run(data, saved);
    let count = week.requests.len();
    let first = summarize::Cli::new(provider);
    let fallback = summarize::Cli::fallback_for(provider);
    let fallback_choice = || fallback.as_ref().map(|cli| (cli.provider, cli as &dyn summarize::Assistant));
    let stages = sender.clone();
    let sink: job::StageSink = Arc::new(move |stage| {
        let _ = stages.unbounded_send(WeekEvent::Stage(stage));
    });
    let on_episode = |index: usize, request: &Request| {
        let _ = sender.unbounded_send(WeekEvent::Episode(index, count, format!("{}  ·  «{}»", request.show.title, request.episode.title)));
    };
    let queue = job::run_queue(data, &week.span, &week.requests, (provider, &first), fallback_choice(), &on_episode, sink);
    if let Some(stop) = queue.stopped {
        return Err(stop);
    }
    let _ = sender.unbounded_send(WeekEvent::Building);
    let built = weekly::build(data, &week, saved, (provider, &first), fallback_choice()).map_err(|error| {
        log::error!("weekly report: {error:#}");
        Failure::Other
    })?;
    if let Some(stop) = built.stopped {
        return Err(stop);
    }
    let pdf = built.pdf.ok_or(Failure::Other)?;
    Ok(WeekDone { pdf, shows: built.shows, left_out: built.missing.len() })
}

fn begin_week(window: &mut Window) -> Task<Message> {
    if !idle(window) {
        return Task::none();
    }
    window.week = Some(WeekState::default());
    window.status = None;
    window.report = None;
    let (sender, receiver) = mpsc::unbounded();
    let data_dir = window.data_dir.clone();
    let saved = window.shows.clone();
    let provider = window.provider;
    std::thread::spawn(move || {
        let outcome = week_job(&data_dir, &saved, provider, &sender);
        let _ = sender.unbounded_send(WeekEvent::Finished(outcome));
    });
    Task::run(receiver, Message::Week)
}

fn week_done_text(done: &WeekDone) -> String {
    let shows = if done.shows == 1 { "1 programa".to_string() } else { format!("{} programas", done.shows) };
    let left_out = match done.left_out {
        0 => String::new(),
        1 => " 1 episodio no se pudo procesar y quedó fuera.".into(),
        many => format!(" {many} episodios no se pudieron procesar y quedaron fuera."),
    };
    format!("Informe semanal listo: {shows}.{left_out}")
}

fn week_stage_text(state: &WeekState, provider: Provider) -> Vec<String> {
    if state.building {
        return vec![WEEK_SUMMARIES.into()];
    }
    match &state.position {
        None => vec![COLLECTING.into()],
        Some((index, count, label)) => {
            let mut lines = vec![format!("Episodio {} de {count}: {label}", index + 1)];
            lines.extend(state.stage.map(|stage| stage_text(stage, provider)));
            lines
        }
    }
}

fn update(window: &mut Window, message: Message) -> Task<Message> {
    if !matches!(message, Message::SearchInput(_) | Message::FeedInput(_) | Message::Run(RunEvent::Stage(Stage::Transcribing(_))) | Message::Week(WeekEvent::Stage(Stage::Transcribing(_))) | Message::Model(ModelEvent::Progress(_))) {
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
            if window.assistant_step.is_some() {
                return Task::none();
            }
            window.provider = provider;
            window.assistant = None;
            window.assistant_notice = None;
            log::info!("provider set to {provider:?}");
            if let Err(error) = job::save_settings(&window.data_dir, &Settings { provider, ..job::settings(&window.data_dir) }) {
                log::error!("saving settings: {error:#}");
            }
            return check_assistant(provider);
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
        Message::DownloadModel => return begin_model_download(window),
        Message::Model(ModelEvent::Progress(progress)) => window.model_download = Some(progress),
        Message::Model(ModelEvent::Finished(outcome)) => {
            window.model_download = None;
            window.model_ready = model::present(&window.data_dir);
            window.model_notice = Some(match outcome {
                Ok(()) => MODEL_READY,
                Err(model::Failure::Interrupted) => MODEL_INTERRUPTED,
                Err(model::Failure::Damaged) => MODEL_DAMAGED,
            });
        }
        Message::AssistantChecked(provider, status) => {
            if provider == window.provider {
                log::info!("{provider:?} status {status:?}");
                window.assistant = Some(status);
            }
        }
        Message::SetUpAssistant => return begin_assistant_setup(window),
        Message::CancelAssistant => window.assistant_cancel.store(true, Ordering::Relaxed),
        Message::Assistant(AssistantEvent::Step(step)) => window.assistant_step = Some(step),
        Message::Assistant(AssistantEvent::Finished(status, outcome)) => {
            window.assistant_step = None;
            window.assistant = Some(status);
            window.assistant_notice = match outcome {
                Ok(()) => Some(format!("{} listo.", provider_name(window.provider))),
                Err(failure) => assistant_failure_text(failure, window.provider),
            };
        }
        Message::StartWeek => return begin_week(window),
        Message::ResumeNow => crate::memory::resume_now(),
        Message::Week(event) => {
            let Some(state) = window.week.as_mut() else { return Task::none() };
            match event {
                WeekEvent::Episode(index, count, label) => {
                    state.position = Some((index, count, label));
                    state.stage = None;
                }
                WeekEvent::Stage(stage) => state.stage = Some(stage),
                WeekEvent::Building => state.building = true,
                WeekEvent::Finished(outcome) => {
                    window.week = None;
                    match outcome {
                        Ok(done) => {
                            log::info!("weekly report written to {}", done.pdf.display());
                            window.status = Some(week_done_text(&done));
                            window.report = Some(done.pdf);
                        }
                        Err(failure) => window.status = Some(failure_text(&failure, window.provider)),
                    }
                    window.unfinished_week = has_unfinished_week(&window.data_dir);
                }
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
    let idle = idle(window) && window.model_ready && window.assistant == Some(assistant::Status::Ready);
    let start = button(START).on_press_maybe((idle && window.picked.is_some()).then_some(Message::Start));
    let week_label = if window.unfinished_week { CONTINUE_WEEK } else { START_WEEK };
    let start_week = button(week_label).on_press_maybe((idle && !window.shows.is_empty()).then_some(Message::StartWeek));

    let setup_rows = model_rows(window).into_iter().chain(assistant_rows(window));
    let mut panel = column(setup_rows).spacing(10);
    panel = panel.push(text(EPISODES_HEADING).size(22)).push(boxed(list).height(Length::Fill)).push(assistant).push(row![start, start_week].spacing(8));
    if let Some(state) = &window.week {
        for line in week_stage_text(state, window.provider) {
            panel = panel.push(text(line));
        }
        if let Some(Stage::Transcribing(percent)) = state.stage {
            panel = panel.push(progress_bar(0.0..=100.0, percent as f32));
        }
    }
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
    if waiting_for_memory(window) {
        panel = panel.push(button(RESUME_NOW).on_press(Message::ResumeNow));
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
    fn window_icon_decodes_to_256_square_rgba() {
        let (rgba, width, height) = decode_rgba(ICON_PNG).unwrap();
        assert_eq!((width, height), (256, 256));
        assert_eq!(rgba.len(), 256 * 256 * 4);
        assert!(window_icon().is_some());
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
        window.model_ready = true;
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
        let _ = update(&mut window, Message::AssistantChecked(Provider::Codex, assistant::Status::Ready));
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
    fn weekly_run_shows_each_episode_and_ends_with_open_pdf() {
        let dir = data_dir("week");
        let mut window = open(dir.clone());
        window.opener = record;
        let _ = update(&mut window, Message::Fetched(Ok(radio_duna())));
        assert!(simulator(view(&window)).find(START_WEEK).is_ok());
        window.week = Some(WeekState::default());
        assert!(simulator(view(&window)).find(COLLECTING).is_ok());
        let _ = update(&mut window, Message::Week(WeekEvent::Episode(2, 26, "Nada Personal  ·  «Episodio de prueba»".into())));
        let _ = update(&mut window, Message::Week(WeekEvent::Stage(Stage::Transcribing(30))));
        {
            let mut ui = simulator(view(&window));
            assert!(ui.find("Episodio 3 de 26: Nada Personal  ·  «Episodio de prueba»").is_ok());
            assert!(ui.find("Transcribiendo: 30 %").is_ok());
        }
        let _ = update(&mut window, Message::Week(WeekEvent::Building));
        assert!(simulator(view(&window)).find(WEEK_SUMMARIES).is_ok());
        let pdf = dir.join("weeks").join("2026-10-26").join(weekly::WEEKLY_FILE);
        let _ = update(&mut window, Message::Week(WeekEvent::Finished(Ok(WeekDone { pdf: pdf.clone(), shows: 13, left_out: 2 }))));
        assert!(window.week.is_none());
        assert!(simulator(view(&window)).find("Informe semanal listo: 13 programas. 2 episodios no se pudieron procesar y quedaron fuera.").is_ok());
        drive(&mut window, |ui| {
            ui.click(OPEN_PDF).unwrap();
        });
        assert_eq!(OPENED.with(|opened| opened.borrow().last().cloned()), Some(pdf));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_stopped_week_offers_to_continue_after_a_reopen() {
        let dir = data_dir("week-stopped");
        let mut window = open(dir.clone());
        window.week = Some(WeekState::default());
        let span = Span::last_seven_days(&chrono::TimeZone::with_ymd_and_hms(&Local, 2026, 11, 2, 7, 0, 0).unwrap());
        job::mark_unfinished_week(&dir, &span, &[Request { show: radio_duna(), episode: episode() }]).unwrap();
        let _ = update(&mut window, Message::Week(WeekEvent::Finished(Err(Failure::Limit))));
        assert!(window.unfinished_week);
        let window = open(dir.clone());
        assert!(window.unfinished_week);
        assert!(simulator(view(&window)).find(CONTINUE_WEEK).is_ok());
        assert_eq!(week_to_run(&dir, &[]).span, span);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn waiting_for_memory_shows_free_memory_and_a_resume_button() {
        let dir = data_dir("memory");
        let mut window = open(dir.clone());
        window.week = Some(WeekState::default());
        let _ = update(&mut window, Message::Week(WeekEvent::Episode(0, 28, "Nada Personal  ·  «Episodio de prueba»".into())));
        assert!(simulator(view(&window)).find(RESUME_NOW).is_err());
        let _ = update(&mut window, Message::Week(WeekEvent::Stage(Stage::WaitingForMemory(1536))));
        assert!(simulator(view(&window)).find("Esperando memoria libre: hay 1,5 GB libres y sigue sola al llegar a 3,0 GB. Cierra otros programas o usa Reanudar ahora.").is_ok());
        drive(&mut window, |ui| {
            ui.click(RESUME_NOW).unwrap();
        });
        let _ = update(&mut window, Message::Week(WeekEvent::Finished(Err(Failure::LowMemory))));
        assert_eq!(window.status.as_deref(), Some(LOW_MEMORY));
    }

    #[test]
    fn a_missing_assistant_offers_setup_and_keeps_runs_off_until_it_is_ready() {
        let dir = data_dir("assistant");
        std::fs::create_dir_all(job::model_path(&dir).parent().unwrap()).unwrap();
        std::fs::write(job::model_path(&dir), b"model").unwrap();
        let mut window = open(dir.clone());
        window.shows = vec![radio_duna()];
        window.provider = Provider::Claude;
        let _ = update(&mut window, Message::AssistantChecked(Provider::Claude, assistant::Status::Missing));
        assert!(simulator(view(&window)).find(assistant_needed_text(Provider::Claude)).is_ok());
        drive(&mut window, |ui| {
            let _ = ui.click(START_WEEK);
        });
        assert!(window.week.is_none());

        window.assistant_step = Some(assistant::Step::Installing);
        assert!(simulator(view(&window)).find("Instalando Claude…").is_ok());
        let _ = update(&mut window, Message::Assistant(AssistantEvent::Step(assistant::Step::SigningIn)));
        assert!(simulator(view(&window)).find(SIGN_IN_NOW).is_ok());
        let _ = update(&mut window, Message::Provider(Provider::Codex));
        assert_eq!(window.provider, Provider::Claude);

        let _ = update(&mut window, Message::Assistant(AssistantEvent::Finished(assistant::Status::SignedOut, Err(assistant::Failure::SignIn))));
        assert!(simulator(view(&window)).find(SIGN_IN_FAILED).is_ok());
        assert!(simulator(view(&window)).find(SIGN_IN).is_ok());

        window.assistant_step = Some(assistant::Step::SigningIn);
        let _ = update(&mut window, Message::Assistant(AssistantEvent::Finished(assistant::Status::Ready, Ok(()))));
        assert!(simulator(view(&window)).find("Claude listo.").is_ok());
        assert!(simulator(view(&window)).find(SET_UP_ASSISTANT).is_err());
        assert_eq!(window.assistant, Some(assistant::Status::Ready));
    }

    #[test]
    fn a_status_for_another_assistant_is_ignored() {
        let mut window = open(data_dir("assistant-switch"));
        window.provider = Provider::Codex;
        let _ = update(&mut window, Message::AssistantChecked(Provider::Claude, assistant::Status::Ready));
        assert_eq!(window.assistant, None);
    }

    #[test]
    fn a_missing_model_offers_the_download_and_keeps_runs_off_until_it_is_ready() {
        let dir = data_dir("model");
        let mut window = open(dir.clone());
        window.shows = vec![radio_duna()];
        assert!(simulator(view(&window)).find(MODEL_NEEDED).is_ok());
        assert!(simulator(view(&window)).find(DOWNLOAD_MODEL).is_ok());
        drive(&mut window, |ui| {
            let _ = ui.click(START_WEEK);
        });
        assert!(window.week.is_none());
        let _ = update(&mut window, Message::Model(ModelEvent::Progress(model::Progress::Bytes(671_088_640, model::MODEL_SIZE))));
        assert!(simulator(view(&window)).find("Descargando el modelo de voz: 640 de 1.549 MB (41 %)").is_ok());
        assert!(simulator(view(&window)).find(DOWNLOAD_MODEL).is_err());
        let _ = update(&mut window, Message::Model(ModelEvent::Progress(model::Progress::Checking)));
        assert!(simulator(view(&window)).find(MODEL_CHECKING).is_ok());
        let _ = update(&mut window, Message::Model(ModelEvent::Finished(Err(model::Failure::Damaged))));
        assert!(simulator(view(&window)).find(MODEL_DAMAGED).is_ok());
        assert!(simulator(view(&window)).find(DOWNLOAD_MODEL).is_ok());
        std::fs::create_dir_all(job::model_path(&dir).parent().unwrap()).unwrap();
        std::fs::write(job::model_path(&dir), b"model").unwrap();
        let _ = update(&mut window, Message::Model(ModelEvent::Finished(Ok(()))));
        assert!(window.model_ready);
        assert!(simulator(view(&window)).find(MODEL_READY).is_ok());
        assert!(simulator(view(&window)).find(DOWNLOAD_MODEL).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_title_carries_the_version() {
        assert_eq!(TITLE, format!("Podcast-Monitor {}", env!("CARGO_PKG_VERSION")));
        assert!(TITLE.ends_with(" 0.1.0"));
    }

    #[test]
    fn failures_read_in_spanish() {
        let missing = failure_text(&Failure::MissingModel(PathBuf::from("datos").join("models").join(job::MODEL_FILE)), Provider::Claude);
        assert_eq!(missing, "Falta el modelo de voz. Usa Descargar modelo y luego Continuar.");
        assert_eq!(failure_text(&Failure::MissingCli, Provider::Claude), "No se encontró Claude. Usa Instalar y conectar y luego Continuar.");
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
