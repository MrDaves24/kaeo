use crate::{command::Command, helpers::display_path};
use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, new_debouncer,
    notify::{EventKind, RecursiveMode, event::ModifyKind},
};
use std::{collections::BTreeSet, path::PathBuf, sync::mpsc::Sender, time::Duration};

/// Only returns on failure, with the error to print
pub fn watch(
    command: Command,
    paths: Vec<PathBuf>,
    canon: Vec<PathBuf>,
    recursive: bool,
) -> String {
    // Open debouncer
    let (tx, rx) = std::sync::mpsc::channel::<Vec<DebouncedEvent>>();
    let debouncer = new_debouncer(Duration::from_millis(500), None, on_event(tx));
    let mut debouncer = match debouncer {
        Ok(d) => d,
        Err(err) => return format!("Failed to create Debouncer\nError : {err:?}"),
    };

    // Watch for change on all paths
    for path in canon.iter() {
        if let Err(err) = debouncer.watch(path, RecursiveMode::Recursive) {
            return format!("Failed to keep an eye on {path:?} for changes\nError : {err:?}");
        }
    }

    // Handle fs events, one run per changed path even if it changed several times in the batch
    for events in rx {
        let changed = changed_paths(events);
        if command.uses_placeholder() {
            let shown: BTreeSet<_> = changed
                .iter()
                .filter_map(|path| display_path(path, &paths, &canon, recursive))
                .collect();
            for (i, path) in shown.iter().enumerate() {
                command.run(Some(path), i == 0);
            }
        } else if !changed.is_empty() {
            command.run(None, true);
        }
    }
    "Stopped receiving file events".into()
}

/// Created, written or renamed paths that still exist
fn changed_paths(events: Vec<DebouncedEvent>) -> BTreeSet<PathBuf> {
    let mut changed = BTreeSet::new();
    for event in events {
        // Editors saving atomically (vim, temp file + rename) give Create or Name, not Data
        // FUTURE : Also kaeo Metadata changes
        let relevant = matches!(
            event.kind,
            EventKind::Create(_) | EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Name(_))
        );
        if relevant {
            // Gone paths: deleted files, temp files renamed by an atomic save
            changed.extend(event.event.paths.into_iter().filter(|p| p.exists()));
        }
    }
    changed
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

#[cfg(test)]
mod tests {
    use super::*;
    use notify_debouncer_full::notify::{
        Event,
        event::{CreateKind, DataChange, MetadataKind},
    };
    use std::time::Instant;

    #[test]
    fn changed() {
        let dir = std::env::temp_dir().join(format!("kaeo watcher {}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (a, b, tmp) = (dir.join("a"), dir.join("b"), dir.join(".a.tmp"));
        std::fs::write(&a, "").unwrap();
        std::fs::write(&b, "").unwrap();

        let event = |kind, path: &PathBuf| {
            DebouncedEvent::new(Event::new(kind).add_path(path.clone()), Instant::now())
        };
        let data = EventKind::Modify(ModifyKind::Data(DataChange::Any));
        let events = vec![
            event(data, &a),
            event(data, &a),
            event(EventKind::Create(CreateKind::File), &b),
            event(EventKind::Create(CreateKind::File), &tmp), // renamed away, gone
            event(
                EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)),
                &dir,
            ),
        ];
        let changed = changed_paths(events);
        std::fs::remove_dir_all(&dir).unwrap();

        assert_eq!(changed, BTreeSet::from([a, b]));
    }
}
