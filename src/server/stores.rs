//! The stores beyond Steam, GOG and Heroic (docs/api/more-stores.md): the
//! Epic sign-in, lists of games added by hand for the stores whose owned
//! library UpNext can't read, and games added one at a time from no store.

use super::players::Me;
use super::{ApiError, ApiResult, AppState, refused};
use crate::db::{OwnGame, PlayerId};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

pub fn routes() -> Router<AppState> {
    use axum::routing::{get, post, put};
    Router::new()
        .route("/epic", get(epic_get).delete(epic_logout))
        .route("/epic/login", post(epic_login))
        .route("/library/by-hand", get(by_hand_get))
        .route("/library/by-hand/{store}", put(by_hand_put))
        .route("/library/added", get(added_get).post(added_post))
        .route("/library/added/{id}", put(added_put).delete(added_delete))
}

#[utoipa::path(get, path = "/epic", tag = "accounts",
    description = "The Epic sign-in: whether it's there, the display name, Epic's sign-in page, and why it ended when Epic \
        turned it down at a sync.",
    responses((status = 200, body = crate::epic::EpicStatus)))]
async fn epic_get(State(state): State<AppState>, Me(me): Me) -> ApiResult<crate::epic::EpicStatus> {
    Ok(Json(state.epic_for(me).status()))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct EpicLoginBody {
    /// What Epic's page showed after signing in: its `authorizationCode`,
    /// or the whole page.
    code: String,
}

#[utoipa::path(post, path = "/epic/login", tag = "accounts",
    description = "Sign in to Epic with the code its sign-in page (`auth_url`) ends on: the `authorizationCode`, or the whole \
        page of JSON. The code works once, for a few minutes.",
    request_body = EpicLoginBody,
    responses((status = 200, body = crate::epic::EpicStatus), (status = 400, description = "No code in it, or Epic turned it down")))]
async fn epic_login(State(state): State<AppState>, Me(me): Me, Json(body): Json<EpicLoginBody>) -> ApiResult<crate::epic::EpicStatus> {
    match state.epic_for(me).login(&body.code).await {
        Ok(s) => Ok(Json(s)),
        Err(e) => Err(ApiError::bad(format!("{e:#}"))),
    }
}

#[utoipa::path(delete, path = "/epic", tag = "accounts",
    description = "Sign out of Epic: the stored sign-in is deleted. The Epic games already read stay.",
    responses((status = 200, body = crate::epic::EpicStatus)))]
async fn epic_logout(State(state): State<AppState>, Me(me): Me) -> ApiResult<crate::epic::EpicStatus> {
    state.epic_for(me).disconnect().await?;
    Ok(Json(state.epic_for(me).status()))
}

/// One store's list of games added by hand.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ByHandList {
    /// The store's source id: `origin`, `uplay`, `battlenet` or `xboxone`.
    store: String,
    /// Its name as people know it: "EA app".
    label: String,
    /// The titles listed, by name.
    titles: Vec<String>,
}

/// Every store a list can be kept for, with its list, empty when there's none.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ByHandLists {
    stores: Vec<ByHandList>,
}

fn lists(state: &AppState, me: PlayerId) -> Result<ByHandLists, ApiError> {
    let mut all = crate::manual::lists(&state.db, me)?;
    let stores = crate::manual::STORES
        .iter()
        .map(|s| ByHandList { store: s.to_string(), label: crate::galaxy::platform_label(s), titles: all.remove(*s).unwrap_or_default() })
        .collect();
    Ok(ByHandLists { stores })
}

#[utoipa::path(get, path = "/library/by-hand", tag = "library",
    description = "The games added by hand, for each store whose owned library UpNext can't read (the EA app, Ubisoft \
        Connect, Battle.net and Xbox): every such store, with its titles, an empty list where none were added.",
    responses((status = 200, body = ByHandLists)))]
async fn by_hand_get(State(state): State<AppState>, Me(me): Me) -> ApiResult<ByHandLists> {
    Ok(Json(lists(&state, me)?))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ByHandBody {
    /// The titles, one a line; an entry may hold several lines. Blank lines,
    /// bullets, trademark signs and repeats are dropped.
    titles: Vec<String>,
}

/// What saving a list did.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ByHandSaved {
    /// Titles on the list now.
    total: usize,
    added: usize,
    /// Titles that were on the list and aren't now: they leave the library.
    removed: usize,
    lists: ByHandLists,
}

#[utoipa::path(put, path = "/library/by-hand/{store}", tag = "library",
    description = "Replace a store's list of games added by hand. A title new to the list joins the library as an owned, \
        uninstalled copy of that store; one left off leaves it. An empty list takes them all out. At most 5,000 titles.",
    params(("store" = String, Path, description = "`origin`, `uplay`, `battlenet` or `xboxone`")),
    request_body = ByHandBody,
    responses((status = 200, body = ByHandSaved), (status = 400, description = "Another store, or too many titles")))]
async fn by_hand_put(
    State(state): State<AppState>,
    Me(me): Me,
    Path(store): Path<String>,
    Json(body): Json<ByHandBody>,
) -> ApiResult<ByHandSaved> {
    let saved = crate::manual::save(&state.db, me, &store, &body.titles).map_err(|e| ApiError::bad(format!("{e:#}")))?;
    Ok(Json(ByHandSaved { total: saved.total, added: saved.added, removed: saved.removed, lists: lists(&state, me)? }))
}

/// The games this player added from no store.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AddedGames {
    games: Vec<OwnGame>,
}

/// A game added or changed, and whether a job started looking it up.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AddedSaved {
    game: OwnGame,
    /// A look-up of this game started; false when it's kept from look-ups,
    /// or another job is running (the next run of Fetch details takes it).
    looking_up: bool,
}

#[utoipa::path(get, path = "/library/added", tag = "library",
    description = "The games this player added by hand from no store (an emulator, a mod, a headset's own store), newest first.",
    responses((status = 200, body = AddedGames)))]
async fn added_get(State(state): State<AppState>, Me(me): Me) -> ApiResult<AddedGames> {
    Ok(Json(AddedGames { games: state.db.own_games(me)? }))
}

/// Look the title up now, for this title only, with the usual sources;
/// whether the job started (another one may be running).
fn look_up(state: &AppState, title: i64) -> bool {
    let opts = crate::enrich::EnrichOptions { titles: vec![title], ..Default::default() };
    super::jobs::start_job(state, crate::jobs::JobKind::Enrich(opts), None).is_ok()
}

/// The checked game, the saved copy and the job looking it up.
fn saved(state: &AppState, me: PlayerId, copy: i64, title: i64, wanted: bool) -> Result<AddedSaved, ApiError> {
    let game = state.db.own_game(me, copy)?.ok_or_else(|| ApiError::not_found("the game vanished"))?;
    Ok(AddedSaved { looking_up: wanted && look_up(state, title), game })
}

fn today() -> chrono::NaiveDate {
    chrono::Utc::now().date_naive()
}

#[utoipa::path(post, path = "/library/added", tag = "library",
    description = "Add a game by hand from no store. It joins this player's library as an owned copy of the `other` source, \
        with where it's from, its cover and Play links, hours and last played as given, and is looked up by name unless \
        `look_up` is false. A Play link must be a launcher's (`steam`, `heroic`, `goggalaxy`, `com.epicgames.launcher`, `uplay`).",
    request_body = crate::added::AddedBody,
    responses((status = 201, body = AddedSaved),
        (status = 400, description = "No name, a field too long, a cover that isn't https, another kind of Play link, a bad \
            date, or the same game from the same place added already")))]
async fn added_post(
    State(state): State<AppState>,
    Me(me): Me,
    Json(body): Json<crate::added::AddedBody>,
) -> Result<(StatusCode, Json<AddedSaved>), ApiError> {
    let g = body.checked(today()).map_err(|r| ApiError::bad(r.0))?;
    let (copy, title) = state.db.add_own_game(me, &g).map_err(refused)?;
    Ok((StatusCode::CREATED, Json(saved(&state, me, copy, title, g.look_up)?)))
}

#[utoipa::path(put, path = "/library/added/{id}", tag = "library",
    description = "Change a game this player added by hand: every field, as for adding it. A new name, or `look_up` turned \
        off, drops its Steam match and what the look-ups found for the old name; with `look_up` on it's looked up again.",
    params(("id" = i64, Path, description = "The copy's id (`copy_id`)")),
    request_body = crate::added::AddedBody,
    responses((status = 200, body = AddedSaved), (status = 400, description = "As for adding it"),
        (status = 404, description = "Not a game this player added")))]
async fn added_put(
    State(state): State<AppState>,
    Me(me): Me,
    Path(id): Path<i64>,
    Json(body): Json<crate::added::AddedBody>,
) -> ApiResult<AddedSaved> {
    let g = body.checked(today()).map_err(|r| ApiError::bad(r.0))?;
    let title =
        state.db.edit_own_game(me, id, &g).map_err(refused)?.ok_or_else(|| ApiError::not_found(format!("no game {id} of yours")))?;
    Ok(Json(saved(&state, me, id, title, g.look_up)?))
}

#[utoipa::path(delete, path = "/library/added/{id}", tag = "library",
    description = "Take a game this player added by hand out of the library. Everyone's marks, ratings and diary on it stay \
        with the title, as for a game a store stops reporting.",
    params(("id" = i64, Path, description = "The copy's id (`copy_id`)")),
    responses((status = 204, description = "Removed"), (status = 404, description = "Not a game this player added")))]
async fn added_delete(State(state): State<AppState>, Me(me): Me, Path(id): Path<i64>) -> Result<StatusCode, ApiError> {
    if !state.db.remove_own_game(me, id)? {
        return Err(ApiError::not_found(format!("no game {id} of yours")));
    }
    Ok(StatusCode::NO_CONTENT)
}
