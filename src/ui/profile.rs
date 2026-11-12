//! Profile cards and roles: their rows, presence and custom status lines,
//! and what a profile offers about friendship.

use super::*;

/// Only mentions get a number; plain unread activity is a dot, because sync
/// does not deliver every message of a room that isn't open.
/// "Ann is", "Ann and Bob are", or "Ann, Bob and 3 others are".
/// What a profile or DM offers for a Discord friendship, as in
/// ProfileData.friend.
pub(super) fn friend_offer(relation: Relation) -> i32 {
    match relation {
        Relation::Stranger => 1,
        Relation::Outgoing => 2,
        Relation::Incoming => 3,
        Relation::Me | Relation::Friend => 0,
    }
}

/// The custom status as the account panel shows it: its words on one line,
/// after its emoji when that's Unicode, and the address of its custom emoji's
/// picture.
pub(super) fn status_line(status: &yapfast_core::chat::CustomStatus) -> (String, Option<String>) {
    let text = one_line(&status.text);
    match (&status.emoji, &status.emoji_id) {
        (_, Some(id)) => (text, yapfast_core::discord::emoji_picture(id)),
        (Some(emoji), None) if text.is_empty() => (emoji.clone(), None),
        (Some(emoji), None) => (format!("{emoji} {text}"), None),
        (None, None) => (text, None),
    }
}

/// A custom status or what someone is doing, as a row shows it: its words on
/// one line, and the address of its custom emoji's picture.
pub(super) fn activity_line(activity: Option<&str>) -> (SharedString, SharedString) {
    let (emoji, words) = activity.map_or((None, ""), yapfast_core::discord::status_parts);
    (one_line(words).into(), emoji.unwrap_or_default().into())
}

/// When ActivityClock's seconds start: 2020 began, in Unix seconds, so
/// Slint's 32-bit integers hold them for decades.
const ACTIVITY_EPOCH: u64 = 1_577_836_800;

/// Now on ActivityClock.
fn activity_now() -> i32 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    i32::try_from(now.saturating_sub(ACTIVITY_EPOCH)).unwrap_or(i32::MAX)
}

/// Someone's activities as their profile's cards show them.
pub(super) fn activity_rows(activities: &yapfast_core::chat::Activities) -> ModelRc<ActivityRow> {
    if activities.is_empty() {
        return ModelRc::default();
    }
    #[cfg(feature = "discord")]
    let rows: Vec<ActivityRow> = activities.iter().map(activity_row).collect();
    #[cfg(not(feature = "discord"))]
    let rows: Vec<ActivityRow> = Vec::new();
    ModelRc::new(VecModel::from(rows))
}

#[cfg(feature = "discord")]
fn activity_row(activity: &yapfast_core::chat::Activity) -> ActivityRow {
    let seconds = |ms: Option<u64>| {
        ms.map_or(0, |ms| {
            i32::try_from((ms / 1000).saturating_sub(ACTIVITY_EPOCH)).unwrap_or(0)
        })
    };
    let text = |text: &Option<Box<str>>| SharedString::from(text.as_deref().unwrap_or_default());
    // A Hang status's emoji goes before its words, as a status's does.
    let name = match &activity.emoji {
        Some(emoji) => format!("{emoji} {}", activity.name),
        None => activity.name.to_string(),
    };
    let (name_emoji, name) = yapfast_core::discord::status_parts(&name);
    let state = match (&activity.state, activity.party) {
        (Some(state), Some((size, most))) => format!("{state} ({size} of {most})"),
        (None, Some((size, most))) => format!("{size} of {most}"),
        (state, None) => state.as_deref().unwrap_or_default().to_owned(),
    };
    // Spotify's card is titled by the song, then by whom and on which album.
    let (name, details, state) = if activity.spotify() {
        let after = |word: &str, text: &Option<Box<str>>| {
            text.as_deref()
                .map_or(String::new(), |text| format!("{word} {text}"))
        };
        (
            activity.details.as_deref().unwrap_or(name).to_owned(),
            after("by", &activity.state),
            after("on", &activity.large_text),
        )
    } else {
        (
            name.to_owned(),
            activity.details.as_deref().unwrap_or_default().to_owned(),
            state,
        )
    };
    ActivityRow {
        heading: activity.heading().to_uppercase().into(),
        name: name.into(),
        name_emoji: name_emoji.unwrap_or_default().into(),
        details: details.into(),
        state: state.into(),
        large_url: text(&activity.large_image),
        large_text: text(&activity.large_text),
        small_url: text(&activity.small_image),
        small_text: text(&activity.small_text),
        start: seconds(activity.start),
        end: seconds(activity.end),
        url: text(&activity.url),
        link: text(&activity.link),
    }
}

/// The game someone plays, which Discord's lists show after a controller,
/// and the line under their name without it: their custom status, else what
/// else they do, such as a song.
pub(super) fn game_and_status(
    line: Option<&str>,
    activities: &yapfast_core::chat::Activities,
) -> (SharedString, Option<String>) {
    #[cfg(feature = "discord")]
    {
        use yapfast_core::chat::ActivityKind;
        let game = activities
            .iter()
            .find(|activity| activity.kind == ActivityKind::Playing);
        let rest = line
            .filter(|line| game.is_none_or(|game| game.line().as_deref() != Some(*line)))
            .map(str::to_owned);
        (
            game.map_or_else(SharedString::default, |game| game.name.as_ref().into()),
            rest,
        )
    }
    #[cfg(not(feature = "discord"))]
    {
        let _ = activities;
        (SharedString::default(), line.map(str::to_owned))
    }
}

/// Whether a line under a name only says what one of the activities does.
pub(super) fn names_an_activity(line: &str, activities: &yapfast_core::chat::Activities) -> bool {
    #[cfg(feature = "discord")]
    return activities
        .iter()
        .any(|activity| activity.line().as_deref() == Some(line));
    #[cfg(not(feature = "discord"))]
    {
        let _ = (line, activities);
        false
    }
}

impl Adapter {
    /// Keeps ActivityClock current, once a second, while a profile on
    /// screen times an activity, and leaves it be otherwise.
    pub(super) fn tick_activities(&mut self, window: &YapfastWindow) {
        let timed = |data: ProfileData| {
            data.activities
                .iter()
                .any(|row| row.start > 0 || row.end > 0)
        };
        // Active Now's cards say how long a game has run, and time songs.
        let active = window.get_friends_open()
            && self
                .models
                .friends_active
                .iter()
                .any(|row| row.since > 0 || row.start > 0 || row.end > 0);
        let shown = active
            || (self.app.state.profile.is_some() && timed(window.get_profile()))
            || (self.app.state.panel_profile.is_some() && timed(window.get_panel_profile()));
        if !shown {
            self.activity_clock.stop();
            return;
        }
        if self.activity_clock.running() {
            return;
        }
        window.global::<ActivityClock>().set_now(activity_now());
        let window = window.as_weak();
        self.activity_clock.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_secs(1),
            move || {
                if let Some(window) = window.upgrade() {
                    window.global::<ActivityClock>().set_now(activity_now());
                }
            },
        );
    }
}

/// The picker's key for the custom status's emoji: the emoji, or `name:id`.
pub(super) fn status_key(status: &yapfast_core::chat::CustomStatus) -> Option<String> {
    match (&status.emoji, &status.emoji_id) {
        (Some(name), Some(id)) => Some(format!("{name}:{id}")),
        (Some(emoji), None) => Some(emoji.clone()),
        _ => None,
    }
}

/// Role rows, each holding a list of its own, compared as `same_list` does.
pub(super) fn same_role_rows(
    old: &ModelRc<ProfileRoleRow>,
    new: ModelRc<ProfileRoleRow>,
) -> ModelRc<ProfileRoleRow> {
    let rows: Vec<ProfileRoleRow> = new
        .iter()
        .enumerate()
        .map(|(index, row)| match old.row_data(index) {
            Some(shown) => ProfileRoleRow {
                roles: same_list(&shown.roles, row.roles),
            },
            None => row,
        })
        .collect();
    same_list(old, ModelRc::new(VecModel::from(rows)))
}

/// The profile to show in place of `shown`, keeping its lists where they're
/// the same; none when nothing changed.
pub(super) fn changed_profile(shown: &ProfileData, mut data: ProfileData) -> Option<ProfileData> {
    data.servers = same_list(&shown.servers, data.servers);
    data.friends = same_list(&shown.friends, data.friends);
    data.activities = same_list(&shown.activities, data.activities);
    data.role_rows = same_role_rows(&shown.role_rows, data.role_rows);
    data.role_rows_large = same_role_rows(&shown.role_rows_large, data.role_rows_large);
    (*shown != data).then_some(data)
}

/// A profile as a card, the full profile or the DM panel shows it.
pub(super) fn profile_data(
    pictures: &mut HashMap<String, slint::Image>,
    state: &AppState,
    view: &ProfileView,
) -> ProfileData {
    let profile = &view.profile;
    let handle = [profile.handle.as_str(), profile.pronouns.as_str()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    // The profile keeps the status's own line breaks. A line that only
    // names an activity goes, as its card says more.
    let (activity_emoji, activity) = profile
        .activity
        .as_deref()
        .filter(|line| !names_an_activity(line, &profile.activities))
        .map_or((None, ""), yapfast_core::discord::status_parts);
    ProfileData {
        id: profile.id.clone().into(),
        name: profile.name.clone().into(),
        initial: initial(&profile.name),
        tint: tint(&profile.name),
        picture: cached_picture(pictures, state, profile.avatar.as_deref()),
        handle: handle.into(),
        banner: cached_picture(pictures, state, profile.banner.as_deref()),
        accent: role_color(profile.accent),
        has_accent: profile.accent.is_some(),
        presence: presence_name(profile.presence).into(),
        activity: activity.into(),
        activity_emoji: activity_emoji.unwrap_or_default().into(),
        activities: activity_rows(&profile.activities),
        bio: markdown(&profile.bio),
        has_bio: !profile.bio.trim().is_empty(),
        role_rows: role_rows(&profile.roles, PROFILE_CARD_WIDTH),
        role_rows_large: role_rows(&profile.roles, PROFILE_LARGE_WIDTH),
        created: profile.created.clone().unwrap_or_default().into(),
        joined: profile.joined.clone().unwrap_or_default().into(),
        servers: ModelRc::new(VecModel::from(
            profile
                .servers
                .iter()
                .map(|server| ProfileServerRow {
                    id: server.id.as_str().into(),
                    name: server.name.as_str().into(),
                    initial: initial(&server.name),
                    picture: cached_picture(pictures, state, server.icon.as_deref()),
                    banner: cached_picture(pictures, state, server.banner.as_deref()),
                    members: server
                        .members
                        .map(|count| count_of(count, "member", "members"))
                        .unwrap_or_default()
                        .into(),
                    channels: count_of(server.channels as u64, "channel", "channels").into(),
                    nick: server.nick.as_deref().unwrap_or_default().into(),
                })
                .collect::<Vec<_>>(),
        )),
        friends: ModelRc::new(VecModel::from(
            profile
                .friends
                .iter()
                .map(|friend| {
                    let known = state.friends.iter().find(|known| known.id == friend.id);
                    ProfileFriendRow {
                        id: friend.id.as_str().into(),
                        name: friend.name.as_str().into(),
                        initial: initial(&friend.name),
                        tint: tint(&friend.id),
                        picture: cached_picture(pictures, state, friend.avatar.as_deref()),
                        presence: presence_name(Some(
                            known
                                .and_then(|known| known.presence)
                                .unwrap_or(Presence::Offline),
                        ))
                        .into(),
                    }
                })
                .collect::<Vec<_>>(),
        )),
        note: if view.loading && !view.kept {
            "Loading the profile...".into()
        } else {
            view.note.clone().unwrap_or_default().into()
        },
        // Matrix IDs start with `@`; Discord user ids are numbers.
        friend: if profile.id.starts_with('@') {
            0
        } else {
            friend_offer(state.relation(&profile.id))
        },
        // Discord's card names them as the profile does.
        message_hint: if state
            .person_choices(&profile.id)
            .contains(&MenuChoice::Message)
        {
            format!("Message @{}", profile.name).into()
        } else {
            Default::default()
        },
    }
}

pub(super) fn presence_name(presence: Option<Presence>) -> &'static str {
    match presence {
        Some(Presence::Online) => "online",
        Some(Presence::Mobile) => "mobile",
        Some(Presence::Idle) => "idle",
        Some(Presence::DoNotDisturb) => "dnd",
        Some(Presence::Offline) => "offline",
        None => "",
    }
}

/// Roles as chips, a row at a time. Slint has no wrapping layout, so chips
/// are measured roughly: Atkinson Hyperlegible Next Medium at 13px averages
/// under 7px a letter, as Satoshi Medium does.
pub(super) fn role_rows(roles: &[ProfileRole], width: f32) -> ModelRc<ProfileRoleRow> {
    const GAP: f32 = 4.;
    // Padding, the colour dot and the space after it.
    const CHROME: f32 = 8. + 10. + 6. + 8.;
    let mut rows: Vec<Vec<yapfast_ui::ProfileRole>> = Vec::new();
    let mut used = width;
    for role in roles {
        let chip = CHROME + 7. * role.name.chars().count() as f32;
        if used + GAP + chip > width || rows.is_empty() {
            rows.push(Vec::new());
            used = chip;
        } else {
            used += GAP + chip;
        }
        if let Some(row) = rows.last_mut() {
            row.push(yapfast_ui::ProfileRole {
                name: role.name.clone().into(),
                color: role_color(role.color),
                has_color: role.color.is_some(),
            });
        }
    }
    ModelRc::new(VecModel::from(
        rows.into_iter()
            .map(|roles| ProfileRoleRow {
                roles: ModelRc::new(VecModel::from(roles)),
            })
            .collect::<Vec<_>>(),
    ))
}

/// A Discord role's RGB colour for Slint.
pub(super) fn role_color(rgb: Option<u32>) -> slint::Color {
    rgb.map_or_else(slint::Color::default, |rgb| {
        slint::Color::from_rgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
    })
}

/// Friends doing something, as Discord's Active Now lists them: playing,
/// listening, streaming or in a server's voice channel, by name.
pub(super) fn active_now_rows(
    friends: &[yapfast_core::chat::Friend],
    in_voice: &HashMap<&str, super::refresh::VoicePlace<'_>>,
) -> Vec<ActiveNowRow> {
    #[cfg(feature = "discord")]
    {
        use yapfast_core::chat::ActivityKind;
        let around = |friend: &&yapfast_core::chat::Friend| {
            !matches!(friend.presence, None | Some(Presence::Offline))
        };
        fn doing(friend: &yapfast_core::chat::Friend) -> Option<&yapfast_core::chat::Activity> {
            friend
                .activities
                .iter()
                .find(|activity| activity.kind != ActivityKind::Hang)
        }
        // How many friends play each game, which a plain game's card says.
        let mut players: HashMap<&str, usize> = HashMap::new();
        for friend in friends.iter().filter(around) {
            if let Some(activity) = doing(friend).filter(|a| a.kind == ActivityKind::Playing) {
                *players.entry(&activity.name).or_default() += 1;
            }
        }
        let mut rows: Vec<(String, ActiveNowRow)> = friends
            .iter()
            .filter(around)
            .filter_map(|friend| {
                let voice = in_voice.get(friend.id.as_str());
                let activity = doing(friend);
                if activity.is_none() && voice.is_none() {
                    return None;
                }
                let mut row = ActiveNowRow {
                    id: friend.id.as_str().into(),
                    name: friend.name.as_str().into(),
                    initial: initial(&friend.name),
                    tint: tint(&friend.id),
                    picture_url: friend.avatar.as_deref().unwrap_or_default().into(),
                    presence: presence_name(friend.presence).into(),
                    voice_room: voice.map_or("", |place| place.room).into(),
                    voice_channel: voice.map_or("", |place| place.channel).into(),
                    voice_server: voice.map_or("", |place| place.server).into(),
                    ..Default::default()
                };
                if let Some(activity) = activity {
                    fill_active_now(&mut row, activity, &players);
                }
                Some((friend.name.to_lowercase(), row))
            })
            .collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        rows.into_iter().map(|(_, row)| row).collect()
    }
    #[cfg(not(feature = "discord"))]
    {
        let _ = (friends, in_voice);
        Vec::new()
    }
}

/// An Active Now card's lines for what someone is doing.
#[cfg(feature = "discord")]
fn fill_active_now(
    row: &mut ActiveNowRow,
    activity: &yapfast_core::chat::Activity,
    players: &HashMap<&str, usize>,
) {
    use yapfast_core::chat::ActivityKind;
    let text = |text: &Option<Box<str>>| SharedString::from(text.as_deref().unwrap_or_default());
    let playing = activity.kind == ActivityKind::Playing;
    row.headline = match activity.kind {
        ActivityKind::Playing => activity.name.to_string(),
        ActivityKind::Listening => activity.heading(),
        _ => activity.line().unwrap_or_else(|| activity.name.to_string()),
    }
    .into();
    if playing {
        row.since = activity_seconds(activity.start);
    }
    // A game's icon, as Discord asks the app for it.
    if let Some(id) = &activity.application_id {
        row.icon_url = format!("discord-app-icon:{id}").into();
    }
    row.large_url = text(&activity.large_image);
    row.small_url = text(&activity.small_image);
    if activity.spotify() {
        song_lines(row, activity);
    } else if activity.details.is_some() || activity.state.is_some() {
        detail_lines(row, activity);
    } else if playing {
        // A game with no details: its name, and how many friends play it.
        let count = players.get(activity.name.as_ref()).copied().unwrap_or(1);
        row.title = activity.name.as_ref().into();
        row.subtitle = format!("{count} {}", if count == 1 { "Person" } else { "People" }).into();
        row.plain = true;
        if row.large_url.is_empty() {
            row.large_url = row.icon_url.clone();
        }
    }
}

/// A time in Unix milliseconds on ActivityClock; 0 for none.
#[cfg(feature = "discord")]
fn activity_seconds(ms: Option<u64>) -> i32 {
    ms.map_or(0, |ms| {
        i32::try_from((ms / 1000).saturating_sub(ACTIVITY_EPOCH)).unwrap_or(0)
    })
}

/// A Spotify song's card: its title, artists and album, and time left.
#[cfg(feature = "discord")]
fn song_lines(row: &mut ActiveNowRow, activity: &yapfast_core::chat::Activity) {
    let text = |text: &Option<Box<str>>| SharedString::from(text.as_deref().unwrap_or_default());
    row.title = activity.details.as_deref().unwrap_or(&activity.name).into();
    row.subtitle = text(&activity.state);
    row.detail = text(&activity.large_text);
    row.end = activity_seconds(activity.end);
}

/// A rich presence's card: its details over its state and party, and the
/// time left, else the time it's run.
#[cfg(feature = "discord")]
fn detail_lines(row: &mut ActiveNowRow, activity: &yapfast_core::chat::Activity) {
    let state = match (&activity.state, activity.party) {
        (Some(state), Some((size, most))) => format!("{state} ({size} of {most})"),
        (None, Some((size, most))) => format!("{size} of {most}"),
        (state, None) => state.as_deref().unwrap_or_default().to_owned(),
    };
    row.title = activity.details.as_deref().unwrap_or(&activity.name).into();
    row.subtitle = state.into();
    row.end = activity_seconds(activity.end);
    if row.end == 0 {
        row.start = activity_seconds(activity.start);
    }
}

/// Someone's Discord looks as a row shows them: their name style's font and
/// effect ids and two colours, and their decoration's and nameplate's
/// pictures' addresses.
pub(super) struct RowLooks {
    pub font: i32,
    pub effect: i32,
    pub color_a: slint::Color,
    pub color_b: slint::Color,
    pub decoration: SharedString,
    pub nameplate: SharedString,
}

pub(super) fn row_looks(tag: Option<&yapfast_core::chat::NameTag>) -> RowLooks {
    let color = |rgb: Option<&u32>| {
        rgb.map_or(slint::Color::default(), |&rgb| {
            slint::Color::from_rgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
        })
    };
    // A style with no colours keeps the name's own colour.
    let effect = tag
        .filter(|tag| !tag.colors.is_empty())
        .map_or(0, |tag| i32::from(tag.effect.max(1)));
    RowLooks {
        font: tag.map_or(0, |tag| i32::from(tag.font)),
        effect,
        color_a: color(tag.and_then(|tag| tag.colors.first())),
        color_b: color(tag.and_then(|tag| tag.colors.get(1).or(tag.colors.first()))),
        decoration: tag
            .and_then(|tag| tag.decoration.as_deref())
            .unwrap_or_default()
            .into(),
        nameplate: tag
            .and_then(|tag| tag.nameplate.as_deref())
            .unwrap_or_default()
            .into(),
    }
}
