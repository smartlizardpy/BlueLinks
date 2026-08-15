use std::time::Instant;

use tauri::{
    utils::config::WebviewUrl,
    webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder},
    Emitter, LogicalPosition, LogicalSize, Manager,
};
use url::Url;

use crate::{
    game::{GameManager, RunSession, TargetProgress},
    persistence::Persistence,
    randomizer::normalize_title,
    types::{FinishPayload, RunOutcome},
};

const HEADER_HEIGHT: f64 = 56.0;
const BLOCKED_NAMESPACES: &[&str] = &[
    "special:",
    "category:",
    "file:",
    "help:",
    "talk:",
    "user:",
    "user talk:",
    "wikipedia:",
    "portal:",
    "template:",
    "template talk:",
    "draft:",
    "module:",
    "mediawiki:",
    "media:",
    "book:",
    "timedtext:",
];

pub fn canonical_article_title(url: &Url) -> Option<String> {
    if url.scheme() != "https"
        || url.host_str() != Some("en.wikipedia.org")
        || !url.path().starts_with("/wiki/")
        || url.query().is_some()
    {
        return None;
    }
    let encoded = &url.path()[6..];
    if encoded.is_empty() {
        return None;
    }
    let decoded = percent_decode(encoded)?;
    let title = decoded
        .replace('_', " ")
        .trim_matches('/')
        .trim()
        .to_string();
    if title.is_empty()
        || BLOCKED_NAMESPACES
            .iter()
            .any(|prefix| title.to_lowercase().starts_with(prefix))
    {
        None
    } else {
        Some(title)
    }
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return None;
            }
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
            output.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output).ok()
}

#[cfg(test)]
pub fn is_allowed_url(url: &Url) -> bool {
    canonical_article_title(url).is_some()
}

fn article_url(title: &str) -> Result<Url, String> {
    let mut url =
        Url::parse("https://en.wikipedia.org/wiki/").map_err(|error| error.to_string())?;
    url.path_segments_mut()
        .map_err(|_| "Invalid Wikipedia base URL")?
        .pop_if_empty()
        .push(&title.replace(' ', "_"));
    Ok(url)
}

fn child_size(width: f64, height: f64, scale: f64) -> LogicalSize<f64> {
    LogicalSize::new(
        (width / scale).round().max(1.0),
        (height / scale - HEADER_HEIGHT).round().max(1.0),
    )
}

fn result_for(run: &RunSession, outcome: RunOutcome) -> FinishPayload {
    FinishPayload {
        start_title: run.challenge.start.title.clone(),
        target_title: run.active_target().title.clone(),
        duration_ms: run
            .started_at
            .map(|instant| instant.elapsed().as_millis() as u64)
            .unwrap_or(0),
        clicks: run.clicks,
        click_limit: run.challenge.click_limit,
        within_click_limit: run.clicks <= run.challenge.click_limit,
        route: run.route.clone(),
        is_personal_best: false,
        success: outcome == RunOutcome::Success,
        outcome,
        mode: run.challenge.mode,
        difficulty: run.challenge.difficulty,
        streak: 0,
        best_streak: 0,
        stage_count: run.stage_count(),
    }
}

fn publish_finish(app: &tauri::AppHandle, persistence: &Persistence, mut payload: FinishPayload) {
    payload.is_personal_best = persistence.record(&mut payload).unwrap_or(false);
    let _ = app.emit_to("main", "game:finish", &payload);
    if let Some(webview) = app.get_webview("wikipedia") {
        let _ = webview.hide();
        let _ = webview.close();
    }
}

fn finish_if_active(
    app: &tauri::AppHandle,
    manager: &GameManager,
    persistence: &Persistence,
    run_id: &str,
    outcome: RunOutcome,
) {
    let payload = manager.inner.lock().ok().and_then(|mut guard| {
        guard.as_mut().and_then(|run| {
            if run.run_id != run_id || run.completed {
                return None;
            }
            run.completed = true;
            Some(result_for(run, outcome))
        })
    });
    if let Some(payload) = payload {
        publish_finish(app, persistence, payload);
    }
}

pub(crate) fn resize_child(app: &tauri::AppHandle, width: f64, height: f64, scale: f64) {
    if let Some(webview) = app.get_webview("wikipedia") {
        let logical = child_size(width, height, scale);
        let logical_width = logical.width;
        let logical_height = logical.height;
        #[cfg(target_os = "linux")]
        {
            // Wry's WebKitGTK set_bounds path can allocate the child at the
            // wrong offset. Move the GtkFixed child directly on Linux only.
            let _ = webview.with_webview(move |platform| {
                use gtk::prelude::{Cast, FixedExt, WidgetExt};
                let inner = platform.inner();
                inner.set_margin_top(0);
                inner.set_size_request(logical_width as i32, logical_height as i32);
                if let Some(parent) = inner.parent() {
                    if let Ok(fixed) = parent.downcast::<gtk::Fixed>() {
                        fixed.move_(&inner, 0, HEADER_HEIGHT as i32);
                        for delay_ms in [16_u64, 100, 300] {
                            let fixed = fixed.clone();
                            let inner = inner.clone();
                            gtk::glib::timeout_add_local_once(
                                std::time::Duration::from_millis(delay_ms),
                                move || {
                                    inner.set_margin_top(0);
                                    inner.set_size_request(
                                        logical_width as i32,
                                        logical_height as i32,
                                    );
                                    fixed.move_(&inner, 0, HEADER_HEIGHT as i32);
                                    inner.queue_allocate();
                                },
                            );
                        }
                    }
                }
            });
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = webview.set_position(LogicalPosition::new(0.0, HEADER_HEIGHT));
            let _ = webview.set_size(LogicalSize::new(logical_width, logical_height));
        }
    }
}

pub async fn open(
    app: tauri::AppHandle,
    manager: GameManager,
    persistence: std::sync::Arc<Persistence>,
) -> Result<(), String> {
    if let Some(old) = app.get_webview("wikipedia") {
        let _ = old.close();
    }
    let (run_id, start_title) = manager
        .inner
        .lock()
        .map_err(|_| "Game state lock was poisoned")?
        .as_ref()
        .map(|run| (run.run_id.clone(), run.challenge.start.title.clone()))
        .ok_or("No prepared run")?;
    let url = article_url(&start_title)?;
    #[cfg(debug_assertions)]
    eprintln!("BlueLink opening Wikipedia URL: {url}");
    let window = app.get_window("main").ok_or("Main window is unavailable")?;
    let inner = window.inner_size().map_err(|error| error.to_string())?;
    let scale = window.scale_factor().map_err(|error| error.to_string())?;
    let size = child_size(inner.width as f64, inner.height as f64, scale);
    let initialization_script = r#"
      (() => {
        const block = e => { if ((e.altKey && (e.key==='ArrowLeft'||e.key==='ArrowRight')) || e.key==='BrowserBack'||e.key==='BrowserForward') { e.preventDefault(); e.stopPropagation(); } };
        addEventListener('keydown', block, true); addEventListener('contextmenu', e => e.preventDefault(), true);
        const style=document.createElement('style'); style.textContent='.vector-search-box,.cdx-search-input,.mw-searchInput,.search-toggle{display:none!important}';
        document.documentElement.appendChild(style);
      })();
    "#;

    let navigation_manager = manager.clone();
    let navigation_app = app.clone();
    let navigation_run = run_id.clone();
    let navigation_persistence = persistence.clone();
    let load_manager = manager.clone();
    let load_app = app.clone();
    let load_run = run_id;
    let load_persistence = persistence.clone();

    let builder = WebviewBuilder::new("wikipedia", WebviewUrl::External(url))
        .initialization_script(initialization_script)
        .on_navigation(move |url| {
            let Some(title) = canonical_article_title(url) else {
                return false;
            };
            let mut finish = None;
            if let Ok(mut guard) = navigation_manager.inner.lock() {
                if let Some(run) = guard.as_mut() {
                    if run.run_id != navigation_run || run.completed {
                        return false;
                    }
                    run.note_navigation_attempt(&title);
                    if run.started_at.is_some()
                        && normalize_title(&title) == normalize_title(&run.active_target().title)
                    {
                        match run.commit_target() {
                            TargetProgress::StageAdvanced(stage) => {
                                let _ = navigation_app.emit_to("main", "game:stage", stage);
                            }
                            TargetProgress::Finished(outcome) => {
                                finish = Some(result_for(run, outcome));
                            }
                        }
                    }
                }
            }
            if let Some(payload) = finish {
                publish_finish(&navigation_app, &navigation_persistence, payload);
            }
            true
        })
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_download(|_, _| false)
        .on_page_load(move |_webview, payload| {
            if payload.event() != PageLoadEvent::Finished {
                return;
            }
            let Some(title) = canonical_article_title(payload.url()) else {
                return;
            };
            let mut started = false;
            let mut update = None;
            let mut failure = None;
            let mut timeout_seconds = None;
            if let Ok(mut guard) = load_manager.inner.lock() {
                if let Some(run) = guard.as_mut() {
                    if run.run_id == load_run && !run.completed {
                        if run.started_at.is_none() {
                            run.started_at = Some(Instant::now());
                            run.current_title = title;
                            run.transaction_open = false;
                            started = true;
                            timeout_seconds = run.challenge.time_limit_seconds;
                        } else {
                            update = Some(run.commit_loaded_article(title));
                            if run.exceeded_click_limit() {
                                run.completed = true;
                                failure = Some(result_for(run, RunOutcome::MaxClicksExceeded));
                            }
                        }
                    }
                }
            }
            if started {
                let _ = load_app.emit_to("main", "game:run_started", ());
                if let Some(seconds) = timeout_seconds {
                    let timeout_app = load_app.clone();
                    let timeout_manager = load_manager.clone();
                    let timeout_persistence = load_persistence.clone();
                    let timeout_run = load_run.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(seconds as u64));
                        finish_if_active(
                            &timeout_app,
                            &timeout_manager,
                            &timeout_persistence,
                            &timeout_run,
                            RunOutcome::TimeExpired,
                        );
                    });
                }
            }
            if let Some(value) = update {
                let _ = load_app.emit_to("main", "game:navigation", value);
            }
            if let Some(payload) = failure {
                publish_finish(&load_app, &load_persistence, payload);
            }
        });
    window
        .add_child(builder, LogicalPosition::new(0.0, HEADER_HEIGHT), size)
        .map_err(|error| error.to_string())?;
    resize_child(&app, inner.width as f64, inner.height as f64, scale);
    Ok(())
}

pub fn close(app: &tauri::AppHandle) {
    if let Some(webview) = app.get_webview("wikipedia") {
        let _ = webview.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_and_fragments_allowed() {
        assert!(is_allowed_url(
            &Url::parse("https://en.wikipedia.org/wiki/Roman_Empire#History").unwrap()
        ));
    }

    #[test]
    fn blocked_destinations_rejected() {
        for url in [
            "https://example.com/wiki/Rome",
            "https://fr.wikipedia.org/wiki/Rome",
            "https://en.wikipedia.org/wiki/Special:Search",
            "https://en.wikipedia.org/wiki/Category:Games",
            "https://en.wikipedia.org/wiki/File:Example.jpg",
            "https://en.wikipedia.org/w/index.php?search=Rome",
            "https://en.wikipedia.org/w/index.php?title=Rome&action=edit",
        ] {
            assert!(!is_allowed_url(&Url::parse(url).unwrap()), "{url}");
        }
    }

    #[test]
    fn canonicalization_handles_encoding_parentheses_and_apostrophes() {
        for (url, expected) in [
            (
                "https://en.wikipedia.org/wiki/Caf%C3%A9_du_Monde#x",
                "Café du Monde",
            ),
            (
                "https://en.wikipedia.org/wiki/Python_(programming_language)",
                "Python (programming language)",
            ),
            (
                "https://en.wikipedia.org/wiki/Writer%27s_cramp",
                "Writer's cramp",
            ),
        ] {
            assert_eq!(
                canonical_article_title(&Url::parse(url).unwrap()).as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn article_urls_have_exactly_one_wiki_slash() {
        assert_eq!(
            article_url("Isaac Newton").unwrap().as_str(),
            "https://en.wikipedia.org/wiki/Isaac_Newton"
        );
    }

    #[test]
    fn child_geometry_preserves_header_at_standard_dpi() {
        let size = child_size(1100.0, 760.0, 1.0);
        assert_eq!((size.width, size.height), (1100.0, 704.0));
    }

    #[test]
    fn child_geometry_uses_logical_pixels_at_windows_hidpi() {
        let size = child_size(2200.0, 1520.0, 2.0);
        assert_eq!((size.width, size.height), (1100.0, 704.0));
    }
}
