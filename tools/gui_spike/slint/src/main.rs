#![windows_subsystem = "windows"]

use gui_spike_common as common;
use slint::{ComponentHandle, Model, SharedString, StandardListViewItem, VecModel};
use std::rc::Rc;
use std::time::Duration;

slint::slint! {
    import { Button, GroupBox, ProgressIndicator, StandardListView, VerticalBox, HorizontalBox } from "std-widgets.slint";

    export component SpikeWindow inherits Window {
        in property <string> shows-heading;
        in property <string> add-label;
        in property <string> remove-label;
        in property <string> run-label;
        in property <string> open-pdf-label;
        in property <string> progress-heading;
        in property <string> accents;
        in property <string> status;
        in property <float> progress;
        in property <[StandardListViewItem]> shows;
        in-out property <int> selected: 0;
        callback add();
        callback remove();
        callback run();
        callback open-pdf();

        VerticalBox {
            Text { text: root.shows-heading; font-size: 20px; font-weight: 700; }
            StandardListView {
                min-height: 90px;
                model: root.shows;
                current-item <=> root.selected;
            }
            HorizontalBox {
                padding: 0;
                alignment: start;
                Button { text: root.add-label; clicked => { root.add(); } }
                Button { text: root.remove-label; enabled: root.selected >= 0; clicked => { root.remove(); } }
            }
            Rectangle { height: 1px; background: #c0c0c0; }
            Text { text: root.progress-heading; font-size: 20px; font-weight: 700; }
            Text { text: root.status; }
            ProgressIndicator { height: 8px; progress: root.progress; }
            HorizontalBox {
                padding: 0;
                alignment: start;
                Button { text: root.run-label; clicked => { root.run(); } }
                Button { text: root.open-pdf-label; clicked => { root.open-pdf(); } }
            }
            Rectangle { height: 1px; background: #c0c0c0; }
            Text { text: root.accents; }
        }
    }
}

fn row((name, publisher): &(String, String)) -> StandardListViewItem {
    StandardListViewItem::from(SharedString::from(format!("{name}  ·  {publisher}")))
}

fn main() -> Result<(), slint::PlatformError> {
    let window = SpikeWindow::new()?;
    let (width, height) = common::WINDOW_SIZE;
    window.window().set_size(slint::LogicalSize::new(width, height));
    window.set_shows_heading(common::SHOWS_HEADING.into());
    window.set_add_label(common::ADD.into());
    window.set_remove_label(common::REMOVE.into());
    window.set_run_label(common::RUN.into());
    window.set_open_pdf_label(common::OPEN_PDF.into());
    window.set_progress_heading(common::PROGRESS_HEADING.into());
    window.set_accents(common::ACCENTS.into());
    window.set_progress(0.45);
    window.set_status(common::status(0.45).into());

    let shows = Rc::new(VecModel::from(common::shows().iter().map(row).collect::<Vec<_>>()));
    window.set_shows(shows.clone().into());

    let added = shows.clone();
    window.on_add(move || added.push(row(&(common::ADDED_SHOW.0.to_string(), common::ADDED_SHOW.1.to_string()))));
    let handle = window.as_weak();
    let removed = shows.clone();
    window.on_remove(move || {
        let window = handle.unwrap();
        let index = window.get_selected();
        if index >= 0 && (index as usize) < removed.row_count() {
            removed.remove(index as usize);
            window.set_selected(-1);
        }
    });
    let handle = window.as_weak();
    window.on_run(move || {
        let window = handle.unwrap();
        let progress = common::advance(window.get_progress());
        window.set_progress(progress);
        window.set_status(common::status(progress).into());
    });
    window.on_open_pdf(common::open_pdf);

    if let Some(path) = common::screenshot_arg() {
        let handle = window.as_weak();
        slint::Timer::single_shot(Duration::from_millis(500), move || {
            let window = handle.unwrap();
            let shot = window.window().take_snapshot().expect("taking snapshot");
            common::save_png(&path, shot.width(), shot.height(), shot.as_bytes());
            slint::quit_event_loop().unwrap();
        });
    }
    window.run()
}
