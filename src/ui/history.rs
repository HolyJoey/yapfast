//! Back and forward, as the arrows at the left of Discord's title bar: the
//! places shown before, and those gone back from.

use super::{Action, Adapter, AppState, DmScreen, Navigation, YapfastWindow};

/// The most places kept to go back to.
const KEPT: usize = 50;

/// Somewhere the window shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Spot {
    Room(String),
    Friends,
    Requests,
    Server(String),
    DirectMessages,
}

#[derive(Default)]
pub(super) struct History {
    back: Vec<Spot>,
    forward: Vec<Spot>,
    here: Option<Spot>,
    /// What the title bar shows, set again only when it changes, as a
    /// refresh runs on every event.
    title: Option<Title>,
}

/// The title bar's place: its name, picture, whether that's round, and glyph.
type Title = (String, String, bool, i32);

impl History {
    /// Notes where the window is now, after a refresh: somewhere new goes
    /// on the way back, and ends the way forward.
    pub(super) fn note(&mut self, state: &AppState, window: &YapfastWindow) {
        let now = place(state);
        if self.here.as_ref() != Some(&now)
            && let Some(previous) = self.here.replace(now)
        {
            self.back.push(previous);
            if self.back.len() > KEPT {
                self.back.remove(0);
            }
            self.forward.clear();
        }
        window.set_can_go_back(!self.back.is_empty());
        window.set_can_go_forward(!self.forward.is_empty());
        let title = title(state);
        if self.title.as_ref() != Some(&title) {
            window.set_title_place(title.0.as_str().into());
            window.set_title_url(title.1.as_str().into());
            window.set_title_round(title.2);
            window.set_title_glyph(title.3);
            self.title = Some(title);
        }
    }
}

/// Where the window is, as Discord's title bar says it: the friends list, a
/// DM with its avatar, or a server with its icon.
fn title(state: &AppState) -> Title {
    let room = state
        .selected
        .as_ref()
        .and_then(|id| state.rooms.iter().find(|room| &room.id == id));
    let (place, url, round, glyph) = match place(state) {
        Spot::Friends => ("Friends".to_owned(), String::new(), false, 1),
        Spot::Requests => ("Message Requests".to_owned(), String::new(), false, 2),
        Spot::DirectMessages => ("Direct Messages".to_owned(), String::new(), false, 0),
        Spot::Room(_) if state.navigation == Navigation::DirectMessages => {
            let room = room.map_or((String::new(), String::new()), |room| {
                (room.name.clone(), room.avatar.clone().unwrap_or_default())
            });
            (room.0, room.1, true, 0)
        }
        Spot::Room(_) | Spot::Server(_) => {
            let server = match &state.navigation {
                Navigation::Server(id) => state.servers.iter().find(|server| &server.id == id),
                Navigation::DirectMessages => None,
            };
            server.map_or((String::new(), String::new(), false, 0), |server| {
                (
                    server.name.clone(),
                    server.icon.clone().unwrap_or_default(),
                    false,
                    0,
                )
            })
        }
    };
    (place, url, round, glyph)
}

/// Where the window is.
fn place(state: &AppState) -> Spot {
    match (&state.navigation, state.dm_screen, &state.selected) {
        (Navigation::DirectMessages, Some(DmScreen::Friends), _) => Spot::Friends,
        (Navigation::DirectMessages, Some(DmScreen::Requests), _) => Spot::Requests,
        (_, _, Some(room)) => Spot::Room(room.clone()),
        (Navigation::Server(server), _, None) => Spot::Server(server.clone()),
        (Navigation::DirectMessages, None, None) => Spot::DirectMessages,
    }
}

impl Adapter {
    /// Goes back a place, or forward with `forward`.
    pub(super) fn travel(&mut self, window: &YapfastWindow, forward: bool) {
        let (from, to) = if forward {
            (&mut self.history.forward, &mut self.history.back)
        } else {
            (&mut self.history.back, &mut self.history.forward)
        };
        let Some(next) = from.pop() else {
            return;
        };
        if let Some(here) = self.history.here.replace(next.clone()) {
            to.push(here);
        }
        match next {
            Spot::Room(room) => self.action(window, Action::Open(room)),
            Spot::Friends => {
                self.action(window, Action::Navigate(Navigation::DirectMessages));
                self.action(window, Action::OpenDmScreen(DmScreen::Friends));
            }
            Spot::Requests => {
                self.action(window, Action::Navigate(Navigation::DirectMessages));
                self.action(window, Action::OpenDmScreen(DmScreen::Requests));
            }
            Spot::Server(server) => {
                self.action(window, Action::Navigate(Navigation::Server(server)))
            }
            Spot::DirectMessages => {
                self.action(window, Action::Navigate(Navigation::DirectMessages));
            }
        }
        // Where it went may not be quite the place, as when a room is gone:
        // that's where the window is now, without a step to go back.
        self.history.here = Some(place(&self.app.state));
        window.set_can_go_back(!self.history.back.is_empty());
        window.set_can_go_forward(!self.history.forward.is_empty());
    }
}
