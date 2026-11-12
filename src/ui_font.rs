//! The UI's typeface. With Discord, gg sans, as Discord's apps draw, once
//! it's downloaded (see `yapfast_discord::ui_font`); until then, and without
//! Discord, Atkinson Hyperlegible Next, which Yapfast carries.

use std::{path::Path, sync::Arc};

use slint::fontique_011::{fontique, shared_collection};

/// The typeface Yapfast carries.
pub const OWN: &str = "Atkinson Hyperlegible Next";

/// The family to draw: gg sans when it's downloaded, which this registers with
/// Slint. Call after selecting the Slint backend, before the window draws.
pub fn family(root: &Path) -> &'static str {
    #[cfg(feature = "discord")]
    if let Some(files) = yapfast_core::yapfast_discord::ui_font::files(root)
        && register(&files)
    {
        return yapfast_core::yapfast_discord::ui_font::FAMILY;
    }
    let _ = root;
    OWN
}

/// Downloads gg sans when it's missing, then switches the window to it. The
/// download stops when the returned task is dropped.
#[cfg(feature = "discord")]
pub fn fetch(
    runtime: &yapfast_core::runtime::BackgroundRuntime,
    root: &Path,
    window: slint::Weak<yapfast_ui::YapfastWindow>,
) -> Option<yapfast_core::runtime::BackgroundTask<()>> {
    use yapfast_core::yapfast_discord::ui_font;
    if ui_font::files(root).is_some() {
        return None;
    }
    let root = root.to_owned();
    Some(runtime.spawn(async move {
        if let Err(error) = ui_font::download(&root).await {
            tracing::info!(%error, "gg sans isn't downloaded; drawing Yapfast's own typeface");
            return;
        }
        tracing::info!("downloaded gg sans");
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(files) = ui_font::files(&root)
                && register(&files)
                && let Some(window) = window.upgrade()
            {
                window.set_ui_font(ui_font::FAMILY.into());
            }
        });
    }))
}

/// Registers the font files with Slint, mapped rather than copied.
fn register(files: &[std::path::PathBuf]) -> bool {
    let mut collection = shared_collection();
    files.iter().all(|path| {
        let Ok(file) = std::fs::File::open(path) else {
            return false;
        };
        // SAFETY: Yapfast writes each file whole beside it and renames it into
        // place, and never changes it after; the map is shared with the font
        // collection for the process's lifetime.
        let Ok(data) = (unsafe { memmap2::Mmap::map(&file) }) else {
            return false;
        };
        !collection
            .register_fonts(fontique::Blob::new(Arc::new(data)), None)
            .is_empty()
    })
}
