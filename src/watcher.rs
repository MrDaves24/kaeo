use crate::{
    command::{Case, Command},
    helpers::find_ancestor,
};
use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, new_debouncer,
    notify::{EventKind, RecursiveMode, event::ModifyKind},
};
use std::{path::PathBuf, sync::mpsc::Sender, time::Duration};

pub fn watch(command: Command, paths: Vec<PathBuf>, recursive: bool) {
    // Open debouncer
    let (tx, rx) = std::sync::mpsc::channel::<Vec<DebouncedEvent>>();
    let debouncer = new_debouncer(Duration::from_millis(500), None, on_event(tx));
    let mut debouncer = match debouncer {
        Ok(d) => d,
        Err(err) => {
            eprintln!("Failed to create Debouncer");
            eprintln!("Error : {err:?}");
            return;
        }
    };

    // Watch for change on all paths
    for path in paths.iter() {
        if let Err(err) = debouncer.watch(path, RecursiveMode::Recursive) {
            eprintln!("Failed to keep an eye on {path:?} for changes");
            eprintln!("Error : {err:?}");
            return;
        }
    }

    // Handle fs events
    for events in rx {
        for event in events {
            let modified = match event.kind {
                EventKind::Modify(modify_kind) => modify_kind,
                // FUTURE : Also kaeo Create
                _ => continue,
            };
            let _data = match modified {
                ModifyKind::Data(data_change) => data_change,
                // FUTURE : Also kaeo Metadata changes
                _ => continue,
            };

            assert_eq!(event.paths.len(), 1);
            if !recursive && command.case() == Case::OnePath {
                let ancestor = find_ancestor(&event.paths[0], &paths);
                command.run(&paths, Some(&ancestor), true);
            } else {
                command.run(&paths, Some(&event.paths[0]), true);
            }
        }
    }
}

fn on_event(tx: Sender<Vec<DebouncedEvent>>) -> impl Fn(DebounceEventResult) {
    move |event: DebounceEventResult| {
        let result = match event {
            Ok(event) => tx.send(event),
            Err(err) => {
                eprintln!("Failed to receive fs event");
                eprintln!("Error : {err:?}");
                return;
            }
        };
        if let Err(err) = result {
            eprintln!("Failed to transmit fs event");
            eprintln!("Error : {err:?}");
        }
    }
}
