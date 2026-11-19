use super::*;
use crate::import_commands::BookEdit;
use nt_application::volume_import::{GenerationId, SeriesId};
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

struct Commands {
    reply: Mutex<mpsc::Receiver<Option<PathBuf>>>,
    imports: AtomicUsize,
}

impl VolumeCommands for Commands {
    fn choose_epub(&self) -> Result<Option<PathBuf>, BookError> {
        self.reply
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| BookError::ChooserUnavailable)
    }

    fn import_epub(&self, _: &BookWorkspace, _: &Path) -> Result<BookChange, BookError> {
        self.imports.fetch_add(1, Ordering::Relaxed);
        Ok(BookChange {
            generation: GenerationId::parse("22222222222222222222222222222222").unwrap(),
            volume: volume(2),
        })
    }

    fn catalog(&self, workspace: &BookWorkspace) -> Result<BookCatalog, BookError> {
        Ok(BookCatalog {
            generation: workspace.generation.clone(),
            volumes: vec![BookSummary {
                id: volume(1),
                title: "Fixture 1".into(),
            }],
        })
    }

    fn view(
        &self,
        _: &BookWorkspace,
        _: &VolumeId,
        _: BookPosition,
    ) -> Result<BookView, BookError> {
        Err(BookError::NotSaved)
    }

    fn preview(&self, _: &BookWorkspace, _: &VolumeId, _: &BookEdit) -> Result<String, BookError> {
        Err(BookError::InvalidEdit)
    }

    fn save(&self, _: &BookWorkspace, _: &VolumeId, _: &BookEdit) -> Result<BookChange, BookError> {
        Err(BookError::InvalidEdit)
    }
}

fn volume(id: u8) -> VolumeId {
    VolumeId::parse(format!("{id:032x}")).unwrap()
}

fn workspace(id: u8) -> BookWorkspace {
    BookWorkspace {
        path: PathBuf::from(format!("fixture-{id}")),
        series: SeriesId::parse(format!("{id:032x}")).unwrap(),
        generation: GenerationId::parse("g0").unwrap(),
    }
}

fn settle(screen: &mut ImportScreen) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while screen.change_job.is_some() || screen.read_job.is_some() || screen.queued_read.is_some() {
        screen.poll();
        assert!(Instant::now() < deadline, "fixture worker did not finish");
        thread::yield_now();
    }
}

#[test]
fn chooser_cancel_keeps_origin_and_chooser_result_after_switch_cannot_import() {
    let (send, reply) = mpsc::sync_channel(1);
    let commands = Arc::new(Commands {
        reply: Mutex::new(reply),
        imports: AtomicUsize::new(0),
    });
    let mut screen = ImportScreen::new(Some(commands.clone()));
    let first = workspace(1);
    screen.open_workspace(1, first.clone());
    screen.open_picker();
    send.send(None).unwrap();
    settle(&mut screen);
    assert!(screen.picker_focused);
    assert!(screen.notice.contains("hủy"));

    screen.open_picker();
    let second = workspace(2);
    screen.open_workspace(2, second.clone());
    send.send(Some(PathBuf::from("巻 1.epub"))).unwrap();
    settle(&mut screen);
    assert_eq!(commands.imports.load(Ordering::Relaxed), 0);
    let (_, active) = screen.workspace.as_ref().unwrap();
    assert_eq!(active.series, second.series);
    assert_eq!(active.generation.as_str(), "g0");
    assert_ne!(active.series, first.series);
}

#[test]
fn choosing_another_volume_and_returning_restores_the_source_position() {
    let mut screen = ImportScreen::new(None);
    screen.select_volume(volume(1));
    let remembered = BookPosition {
        row: 3,
        range: 2,
        unit: 198,
    };
    screen.position = remembered;
    screen.select_volume(volume(2));
    assert_eq!(screen.position, BookPosition::default());
    screen.select_volume(volume(1));
    assert_eq!(screen.position, remembered);
}
