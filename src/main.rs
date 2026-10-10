mod condorcet;
mod html;

use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{FromRequestParts, Path, Query, Request, State},
    http::{header, request::Parts, HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use maud::Markup;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub struct App {
    db: Mutex<Connection>,
    pub base_url: String,
    secure_cookies: bool,
}
type St = State<Arc<App>>;

pub struct Poll {
    pub id: String,
    pub admin_key: String,
    pub title: String,
    pub options: Vec<String>,
    pub invite: bool,
    pub live: bool,
    pub editable: bool,
    pub seed: Vec<u8>,
    pub closed: bool,
}

pub struct Voter {
    pub name: String,
    pub token: String,
    pub voted: bool,
}

pub struct Results {
    pub ballots: usize,
    pub duels: Vec<Vec<u32>>,
    pub winner: Option<usize>,
    pub probs: Vec<f64>,
    /// Set once the poll is closed: (revealed seed, u in [0,1), drawn option).
    pub draw: Option<(String, f64, usize)>,
}

pub const MIN_LIVE_BALLOTS: usize = 3;

#[derive(Clone, Copy, PartialEq)]
pub enum Lang {
    Fr,
    En,
}

impl Lang {
    pub fn t(self, fr: &'static str, en: &'static str) -> &'static str {
        if self == Lang::Fr { fr } else { en }
    }
    pub fn code(self) -> &'static str {
        self.t("fr", "en")
    }
}

fn query_lang(q: Option<&str>) -> Option<Lang> {
    q?.split('&').find_map(|kv| match kv {
        "lang=fr" => Some(Lang::Fr),
        "lang=en" => Some(Lang::En),
        _ => None,
    })
}

impl<S: Send + Sync> FromRequestParts<S> for Lang {
    type Rejection = Infallible;
    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Infallible> {
        let h = &parts.headers;
        Ok(query_lang(parts.uri.query())
            .or_else(|| match cookie(h, "lang").as_deref() {
                Some("fr") => Some(Lang::Fr),
                Some("en") => Some(Lang::En),
                _ => None,
            })
            .unwrap_or_else(|| {
                let accept = h.get(header::ACCEPT_LANGUAGE).and_then(|v| v.to_str().ok()).unwrap_or("");
                if accept.trim_start().starts_with("fr") { Lang::Fr } else { Lang::En }
            }))
    }
}

/// Remembers an explicit `?lang=` choice in a cookie.
async fn remember_lang(req: Request, next: Next) -> Response {
    let lang = query_lang(req.uri().query());
    let mut res = next.run(req).await;
    if let Some(l) = lang {
        let c = format!("lang={}; Path=/; Max-Age=31536000; SameSite=Lax", l.code());
        res.headers_mut().append(header::SET_COOKIE, HeaderValue::from_str(&c).unwrap());
    }
    res
}

fn cookie(h: &HeaderMap, name: &str) -> Option<String> {
    h.get_all(header::COOKIE).iter().filter_map(|v| v.to_str().ok()).flat_map(|v| v.split(';')).find_map(|kv| {
        let (k, v) = kv.trim().split_once('=')?;
        (k == name).then(|| v.to_string())
    })
}

pub struct AppError(StatusCode, Markup);
type Res<T = Response> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        eprintln!("db error: {e}");
        AppError(StatusCode::INTERNAL_SERVER_ERROR, html::error(Lang::En, "Internal error"))
    }
}

fn bad(l: Lang, msg: &str) -> AppError {
    AppError(StatusCode::BAD_REQUEST, html::error(l, msg))
}

fn not_found(l: Lang) -> AppError {
    AppError(StatusCode::NOT_FOUND, html::error(l, l.t("Vote introuvable.", "Poll not found.")))
}

fn rand_id(n: usize) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    (0..n).map(|_| A[(rand::random::<u8>() & 63) as usize] as char).collect()
}

fn sha256_hex(data: &[u8]) -> String {
    Sha256::digest(data).iter().map(|b| format!("{b:02x}")).collect()
}

/// Links a ballot to its voter's secret token without storing the token next to the ballot.
fn edit_key(token: &str) -> String {
    sha256_hex(format!("ballot:{token}").as_bytes())
}

pub fn commitment(seed: &[u8]) -> String {
    sha256_hex(seed)
}

fn seed_to_u(seed: &[u8]) -> f64 {
    u64::from_be_bytes(seed[..8].try_into().unwrap()) as f64 / 2f64.powi(64)
}

fn open_db(path: &str) -> rusqlite::Result<Connection> {
    let c = Connection::open(path)?;
    c.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA busy_timeout=5000;
         CREATE TABLE IF NOT EXISTS polls (
           id TEXT PRIMARY KEY, admin_key TEXT NOT NULL, title TEXT NOT NULL, options TEXT NOT NULL,
           invite INTEGER NOT NULL, live INTEGER NOT NULL, editable INTEGER NOT NULL,
           seed BLOB NOT NULL, closed INTEGER NOT NULL DEFAULT 0, created_at INTEGER NOT NULL);
         CREATE TABLE IF NOT EXISTS voters (
           poll_id TEXT NOT NULL, name TEXT NOT NULL, token TEXT NOT NULL, voted INTEGER NOT NULL DEFAULT 0,
           PRIMARY KEY (poll_id, name), UNIQUE (poll_id, token)) WITHOUT ROWID;
         -- No name, no timestamp, random id: ballots cannot be matched to voters.
         CREATE TABLE IF NOT EXISTS ballots (
           id TEXT PRIMARY KEY, poll_id TEXT NOT NULL, ranks TEXT NOT NULL, edit_key TEXT) WITHOUT ROWID;
         CREATE INDEX IF NOT EXISTS ballots_poll ON ballots(poll_id);",
    )?;
    Ok(c)
}

fn load_poll(c: &Connection, id: &str) -> rusqlite::Result<Option<Poll>> {
    c.query_row(
        "SELECT id, admin_key, title, options, invite, live, editable, seed, closed FROM polls WHERE id = ?",
        [id],
        |r| {
            Ok(Poll {
                id: r.get(0)?,
                admin_key: r.get(1)?,
                title: r.get(2)?,
                options: r.get::<_, String>(3)?.split('\n').map(String::from).collect(),
                invite: r.get(4)?,
                live: r.get(5)?,
                editable: r.get(6)?,
                seed: r.get(7)?,
                closed: r.get(8)?,
            })
        },
    )
    .optional()
}

fn find_voter(c: &Connection, poll: &str, token: &str) -> rusqlite::Result<Option<Voter>> {
    c.query_row("SELECT name, token, voted FROM voters WHERE poll_id = ? AND token = ?", [poll, token], |r| {
        Ok(Voter { name: r.get(0)?, token: r.get(1)?, voted: r.get(2)? })
    })
    .optional()
}

fn voters(c: &Connection, poll: &str) -> rusqlite::Result<Vec<Voter>> {
    let mut s = c.prepare("SELECT name, token, voted FROM voters WHERE poll_id = ? ORDER BY name COLLATE NOCASE")?;
    s.query_map([poll], |r| Ok(Voter { name: r.get(0)?, token: r.get(1)?, voted: r.get(2)? }))?.collect()
}

fn parse_ranks(s: &str, n: usize) -> Option<Vec<u32>> {
    let r: Vec<u32> = s.split(',').map(|x| x.trim().parse().ok()).collect::<Option<_>>()?;
    (r.len() == n).then_some(r)
}

/// Results, or None when they must stay hidden (not closed and live results disabled or too few ballots).
fn results(c: &Connection, p: &Poll) -> rusqlite::Result<Option<Results>> {
    let mut s = c.prepare("SELECT ranks FROM ballots WHERE poll_id = ?")?;
    let n = p.options.len();
    let ballots: Vec<Vec<u32>> = s
        .query_map([&p.id], |r| r.get::<_, String>(0))?
        .filter_map(|r| r.ok().and_then(|s| parse_ranks(&s, n)))
        .collect();
    if !p.closed && (!p.live || ballots.len() < MIN_LIVE_BALLOTS) {
        return Ok(None);
    }
    let duels = condorcet::duels(n, &ballots);
    let winner = condorcet::condorcet_winner(&duels);
    let probs = condorcet::maximal_lottery(&duels);
    let draw = (p.closed && !ballots.is_empty()).then(|| {
        let u = seed_to_u(&p.seed);
        (p.seed.iter().map(|b| format!("{b:02x}")).collect(), u, condorcet::draw(&probs, u))
    });
    Ok(Some(Results { ballots: ballots.len(), duels, winner, probs, draw }))
}

fn ballot_count(c: &Connection, poll: &str) -> rusqlite::Result<usize> {
    c.query_row("SELECT COUNT(*) FROM ballots WHERE poll_id = ?", [poll], |r| r.get::<_, i64>(0)).map(|n| n as usize)
}

async fn home(l: Lang) -> Markup {
    html::home(l)
}

#[derive(Deserialize)]
struct CreateForm {
    title: String,
    options: String,
    mode: String,
    #[serde(default)]
    voters: String,
    results: String,
    ballots: String,
}

fn lines(s: &str) -> Vec<String> {
    s.lines().map(|x| x.split_whitespace().collect::<Vec<_>>().join(" ")).filter(|x| !x.is_empty()).collect()
}

fn has_duplicates(v: &[String]) -> bool {
    let mut seen = std::collections::HashSet::new();
    !v.iter().all(|x| seen.insert(x.to_lowercase()))
}

async fn create(l: Lang, State(app): St, Form(f): Form<CreateForm>) -> Res {
    let title = f.title.trim();
    let options = lines(&f.options);
    let invite = f.mode == "invite";
    let names = if invite { lines(&f.voters) } else { vec![] };
    if title.is_empty() || title.chars().count() > 200 {
        return Err(bad(l, l.t("Le titre est obligatoire (200 caractères max).", "A title is required (200 characters max).")));
    }
    if options.len() < 2 || options.len() > 30 || options.iter().any(|o| o.chars().count() > 200) {
        return Err(bad(l, l.t("Il faut entre 2 et 30 options (200 caractères max chacune).", "You need between 2 and 30 options (200 characters max each).")));
    }
    if has_duplicates(&options) {
        return Err(bad(l, l.t("Deux options portent le même nom.", "Two options have the same name.")));
    }
    if invite && (names.is_empty() || names.len() > 1000 || names.iter().any(|n| n.chars().count() > 100)) {
        return Err(bad(l, l.t("Le mode sur invitation demande entre 1 et 1000 votants (100 caractères max par nom).", "Invitation mode needs between 1 and 1000 voters (100 characters max per name).")));
    }
    if has_duplicates(&names) {
        return Err(bad(l, l.t("Deux votants portent le même nom.", "Two voters have the same name.")));
    }

    let (id, key) = (rand_id(10), rand_id(24));
    let seed: [u8; 32] = rand::random();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let mut c = app.db.lock().unwrap();
    let tx = c.transaction()?;
    tx.execute(
        "INSERT INTO polls (id, admin_key, title, options, invite, live, editable, seed, created_at) VALUES (?,?,?,?,?,?,?,?,?)",
        params![id, key, title, options.join("\n"), invite, f.results == "live", f.ballots == "editable", &seed[..], now],
    )?;
    for n in &names {
        tx.execute("INSERT INTO voters (poll_id, name, token) VALUES (?,?,?)", params![id, n, rand_id(24)])?;
    }
    tx.commit()?;
    Ok(Redirect::to(&format!("/p/{id}/admin/{key}")).into_response())
}

#[derive(Deserialize)]
struct TokenQuery {
    t: Option<String>,
    ok: Option<String>,
}

fn voter_token(p: &Poll, q: Option<String>, h: &HeaderMap) -> Option<String> {
    if p.invite { q } else { cookie(h, "v") }
}

async fn poll_page(l: Lang, State(app): St, Path(id): Path<String>, Query(q): Query<TokenQuery>, h: HeaderMap) -> Res<Markup> {
    let c = app.db.lock().unwrap();
    let p = load_poll(&c, &id)?.ok_or(not_found(l))?;
    let voter = match voter_token(&p, q.t, &h) {
        Some(t) => find_voter(&c, &p.id, &t)?,
        None => None,
    };
    let my_ranks = match &voter {
        Some(v) if v.voted && p.editable => c
            .query_row("SELECT ranks FROM ballots WHERE poll_id = ? AND edit_key = ?", [&p.id, &edit_key(&v.token)], |r| r.get(0))
            .optional()?,
        _ => None,
    };
    let participants: Vec<String> = voters(&c, &p.id)?.into_iter().filter(|v| v.voted).map(|v| v.name).collect();
    let count = ballot_count(&c, &p.id)?;
    Ok(html::poll(l, &p, voter.as_ref(), my_ranks, &participants, count, results(&c, &p)?.as_ref(), q.ok.is_some()))
}

#[derive(Deserialize)]
struct VoteForm {
    ranking: String,
    #[serde(default)]
    name: String,
    t: Option<String>,
}

async fn vote(l: Lang, State(app): St, Path(id): Path<String>, h: HeaderMap, Form(f): Form<VoteForm>) -> Res {
    let mut c = app.db.lock().unwrap();
    let tx = c.transaction()?;
    let p = load_poll(&tx, &id)?.ok_or(not_found(l))?;
    if p.closed {
        return Err(bad(l, l.t("Ce vote est clos.", "This poll is closed.")));
    }
    let ranks = parse_ranks(&f.ranking, p.options.len()).ok_or(bad(l, l.t("Classement invalide.", "Invalid ranking.")))?;
    let ranks = ranks.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(",");

    let mut new_cookie = None;
    let voter = match voter_token(&p, f.t, &h) {
        Some(t) => find_voter(&tx, &p.id, &t)?,
        None => None,
    };
    let voter = match voter {
        Some(v) => v,
        None if p.invite => {
            return Err(AppError(StatusCode::FORBIDDEN, html::error(l, l.t("Lien d'invitation invalide.", "Invalid invitation link."))));
        }
        None => {
            let name = lines(&f.name).join(" ");
            if name.is_empty() || name.chars().count() > 100 {
                return Err(bad(l, l.t("Indiquez votre nom (100 caractères max).", "Please enter your name (100 characters max).")));
            }
            let taken: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM voters WHERE poll_id = ? AND name = ? COLLATE NOCASE)",
                [&p.id, &name],
                |r| r.get(0),
            )?;
            if taken {
                return Err(bad(l, l.t("Ce nom a déjà voté. Choisissez-en un autre.", "This name has already voted. Please choose another one.")));
            }
            let token = rand_id(24);
            tx.execute("INSERT INTO voters (poll_id, name, token) VALUES (?,?,?)", params![p.id, name, token])?;
            new_cookie = Some(token.clone());
            Voter { name, token, voted: false }
        }
    };

    let key = edit_key(&voter.token);
    let updated = voter.voted
        && p.editable
        && tx.execute("UPDATE ballots SET ranks = ? WHERE poll_id = ? AND edit_key = ?", params![ranks, p.id, key])? > 0;
    if voter.voted && !p.editable {
        return Err(bad(l, l.t("Vous avez déjà voté, et ce vote n'est pas modifiable.", "You have already voted, and ballots cannot be changed in this poll.")));
    }
    if !updated {
        tx.execute(
            "INSERT INTO ballots (id, poll_id, ranks, edit_key) VALUES (?,?,?,?)",
            params![rand_id(22), p.id, ranks, p.editable.then_some(key)],
        )?;
        tx.execute("UPDATE voters SET voted = 1 WHERE poll_id = ? AND token = ?", [&p.id, &voter.token])?;
    }
    tx.commit()?;

    let to = if p.invite { format!("/p/{}?t={}&ok=1", p.id, voter.token) } else { format!("/p/{}?ok=1", p.id) };
    let mut res = Redirect::to(&to).into_response();
    if let Some(t) = new_cookie {
        let secure = if app.secure_cookies { "; Secure" } else { "" };
        let c = format!("v={t}; Path=/p/{}; Max-Age=31536000; HttpOnly; SameSite=Lax{secure}", p.id);
        res.headers_mut().append(header::SET_COOKIE, HeaderValue::from_str(&c).unwrap());
    }
    Ok(res)
}

fn load_admin(c: &Connection, l: Lang, id: &str, key: &str) -> Res<Poll> {
    load_poll(c, id)?.filter(|p| p.admin_key == key).ok_or(not_found(l))
}

async fn admin(l: Lang, State(app): St, Path((id, key)): Path<(String, String)>) -> Res<Markup> {
    let c = app.db.lock().unwrap();
    let p = load_admin(&c, l, &id, &key)?;
    Ok(html::admin(l, &app.base_url, &p, &voters(&c, &p.id)?, ballot_count(&c, &p.id)?, results(&c, &p)?.as_ref()))
}

async fn close(l: Lang, State(app): St, Path((id, key)): Path<(String, String)>) -> Res {
    let c = app.db.lock().unwrap();
    let p = load_admin(&c, l, &id, &key)?;
    c.execute("UPDATE polls SET closed = 1 WHERE id = ?", [&p.id])?;
    Ok(Redirect::to(&format!("/p/{id}/admin/{key}")).into_response())
}

async fn sortable() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript"), (header::CACHE_CONTROL, "public, max-age=86400")],
        include_str!("../static/Sortable.min.js"),
    )
}

#[tokio::main]
async fn main() {
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "condorcet.db".into());
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let app = Arc::new(App {
        db: Mutex::new(open_db(&db_path).expect("cannot open database")),
        secure_cookies: base_url.starts_with("https://"),
        base_url,
    });
    let router = Router::new()
        .route("/", get(home).post(create))
        .route("/p/{id}", get(poll_page))
        .route("/p/{id}/vote", post(vote))
        .route("/p/{id}/admin/{key}", get(admin))
        .route("/p/{id}/admin/{key}/close", post(close))
        .route("/static/Sortable.min.js", get(sortable))
        .route("/up", get(|| async { "OK" }))
        .layer(middleware::from_fn(remember_lang))
        .with_state(app);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    println!("listening on :{port}");
    axum::serve(listener, router).await.unwrap();
}
