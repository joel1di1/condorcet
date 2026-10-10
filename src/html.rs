use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::{commitment, Lang, Poll, Results, Voter, MIN_LIVE_BALLOTS};

const CSS: &str = include_str!("../static/app.css");
const VIDEO: &str = "https://www.youtube.com/watch?v=wKimU8jy2a8";

// Class sets adapted from TailwindPlus Application UI.
const CARD: &str = "rounded-xl bg-white p-6 shadow-xs ring-1 ring-gray-900/5 sm:p-8";
const H2: &str = "text-base/7 font-semibold text-gray-900";
const MUTED: &str = "text-sm/6 text-gray-600";
const LABEL: &str = "block text-sm/6 font-medium text-gray-900";
const INPUT: &str = "mt-2 block w-full rounded-md bg-white px-3 py-1.5 text-base text-gray-900 outline-1 -outline-offset-1 outline-gray-300 placeholder:text-gray-400 focus:outline-2 focus:-outline-offset-2 focus:outline-indigo-600 sm:text-sm/6";
const RADIO: &str = "mt-1 size-4 shrink-0 accent-indigo-600 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600";
const BTN: &str = "inline-flex items-center justify-center gap-x-2 rounded-md bg-indigo-600 px-3.5 py-2.5 text-sm font-semibold text-white shadow-xs hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600";
const BTN2: &str = "inline-flex items-center justify-center gap-x-1.5 rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 shadow-xs ring-1 ring-inset ring-gray-300 hover:bg-gray-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600";
const BTN_DANGER: &str = "inline-flex items-center justify-center rounded-md bg-red-600 px-3.5 py-2.5 text-sm font-semibold text-white shadow-xs hover:bg-red-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-600";
const ICON_BTN: &str = "rounded-md p-2 text-gray-500 hover:bg-gray-100 hover:text-gray-900 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 disabled:cursor-not-allowed disabled:opacity-25 disabled:hover:bg-transparent";
const LINK: &str =
    "font-semibold text-indigo-600 hover:text-indigo-500 underline-offset-2 hover:underline";

fn badge(color: &str, text: &str) -> Markup {
    let (bg, dot) = match color {
        "green" => ("bg-green-100 text-green-700", "fill-green-500"),
        "red" => ("bg-red-100 text-red-700", "fill-red-500"),
        "indigo" => ("bg-indigo-100 text-indigo-700", "fill-indigo-500"),
        _ => ("bg-gray-100 text-gray-600", "fill-gray-400"),
    };
    html! {
        span class={"inline-flex items-center gap-x-1.5 rounded-full px-2 py-1 text-xs font-medium " (bg)} {
            svg viewBox="0 0 6 6" aria-hidden="true" class={"size-1.5 " (dot)} { circle r="3" cx="3" cy="3" {} }
            (text)
        }
    }
}

fn alert(kind: &str, body: Markup) -> Markup {
    let cls = match kind {
        "success" => "bg-green-50 text-green-800 ring-green-600/20",
        "warning" => "bg-yellow-50 text-yellow-800 ring-yellow-600/20",
        _ => "bg-indigo-50 text-indigo-800 ring-indigo-600/20",
    };
    html! { div class={"rounded-md p-4 text-sm/6 ring-1 ring-inset " (cls)} role=[(kind == "success").then_some("status")] { (body) } }
}

fn page(l: Lang, title: &str, body: Markup) -> Markup {
    let switch = |code: &str, label: &str, native: &str| {
        let current = l.code() == code;
        html! {
            a href={"?lang=" (code)} lang=(code) hreflang=(code) aria-current=[current.then_some("true")]
              class={"rounded-md px-2 py-1 text-sm font-medium focus-visible:outline-2 focus-visible:outline-indigo-600 " (if current { "bg-indigo-50 text-indigo-700" } else { "text-gray-600 hover:text-gray-900" })}
              onclick={"var u=new URL(location.href);u.searchParams.set('lang','" (code) "');location.href=u;return false"} {
                span aria-hidden="true" { (label) } span class="sr-only" { (native) }
            }
        }
    };
    html! {
        (DOCTYPE)
        html lang=(l.code()) class="h-full bg-gray-50" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="referrer" content="no-referrer";
                meta name="robots" content="noindex";
                title { (title) " · Condorcet" }
                link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><text y='.9em' font-size='90'>🗳️</text></svg>";
                style { (PreEscaped(CSS)) }
            }
            body class="h-full text-gray-900 antialiased" {
                a href="#main" class="sr-only focus:not-sr-only focus:absolute focus:top-2 focus:left-2 focus:z-50 focus:rounded-md focus:bg-white focus:px-3 focus:py-2 focus:shadow" {
                    (l.t("Aller au contenu", "Skip to content"))
                }
                header class="border-b border-gray-200 bg-white" {
                    div class="mx-auto flex max-w-3xl items-center justify-between px-4 py-3 sm:px-6" {
                        a href="/" class="flex items-center gap-x-2 rounded-md text-base font-semibold text-gray-900 focus-visible:outline-2 focus-visible:outline-indigo-600" {
                            span aria-hidden="true" class="flex size-8 items-center justify-center rounded-lg bg-indigo-600 text-white" { "🗳" }
                            "Condorcet"
                        }
                        nav aria-label=(l.t("Langue", "Language")) class="flex gap-x-1" {
                            (switch("fr", "FR", "Français"))
                            (switch("en", "EN", "English"))
                        }
                    }
                }
                main id="main" class="mx-auto max-w-3xl space-y-6 px-4 py-8 sm:px-6 sm:py-12" { (body) }
                footer class="mx-auto max-w-3xl px-4 pb-10 text-center text-xs/5 text-gray-500 sm:px-6" {
                    (l.t("Scrutin de Condorcet randomisé, sans compte. ", "Randomized Condorcet voting, no account needed. "))
                    a class="underline hover:text-gray-700" href=(VIDEO) { (l.t("Comprendre la méthode (vidéo Science4All)", "Learn the method (Science4All video, French)")) }
                }
            }
        }
    }
}

pub fn error(l: Lang, msg: &str) -> Markup {
    page(
        l,
        l.t("Erreur", "Error"),
        html! {
            div class=(CARD) role="alert" {
                h1 class="text-lg font-semibold text-gray-900" { (l.t("Oups", "Oops")) }
                p class={"mt-2 " (MUTED)} { (msg) }
                div class="mt-6 flex gap-3" {
                    button type="button" class=(BTN2) onclick="history.back()" { "← " (l.t("Retour", "Back")) }
                    a href="/" class=(BTN2) { (l.t("Accueil", "Home")) }
                }
            }
        },
    )
}

fn radio(name: &str, value: &str, checked: bool, title: &str, desc: &str) -> Markup {
    let id = format!("{name}-{value}");
    html! {
        div class="flex gap-x-3" {
            input type="radio" id=(id) name=(name) value=(value) checked[checked] class=(RADIO) aria-describedby={(id) "-desc"};
            div class="text-sm/6" {
                label for=(id) class="font-medium text-gray-900" { (title) }
                p id={(id) "-desc"} class="text-gray-600" { (desc) }
            }
        }
    }
}

fn fieldset(legend: &str, body: Markup) -> Markup {
    html! {
        fieldset {
            legend class=(LABEL) { (legend) }
            div class="mt-3 space-y-4" { (body) }
        }
    }
}

pub fn home(l: Lang) -> Markup {
    page(
        l,
        l.t("Nouveau vote", "New poll"),
        html! {
            div {
                h1 class="text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl" {
                    (l.t("Votez en classant, simplement.", "Vote by ranking, simply."))
                }
                p class="mt-4 text-base/7 text-gray-600" {
                    (l.t(
                        "Chacun classe les options par ordre de préférence. L'option qui bat toutes les autres en duel gagne (gagnant de Condorcet). S'il n'y en a pas, le gagnant est tiré au sort selon la loterie optimale du scrutin de Condorcet randomisé. Pas de compte, juste des liens.",
                        "Everyone ranks the options. The option that beats every other one head-to-head wins (Condorcet winner). If there is none, the winner is drawn from the optimal lottery of randomized Condorcet voting. No account, just links."
                    ))
                }
            }
            form method="post" action="/" class={"group/form space-y-8 " (CARD)} {
                div {
                    label for="title" class=(LABEL) { (l.t("Question", "Question")) }
                    input id="title" name="title" type="text" required maxlength="200" class=(INPUT)
                        placeholder=(l.t("Où allons-nous dîner vendredi ?", "Where do we go for dinner on Friday?"));
                }
                div {
                    label for="options" class=(LABEL) { (l.t("Options", "Options")) }
                    p id="options-desc" class={"mt-1 " (MUTED)} { (l.t("Une option par ligne (2 à 30).", "One option per line (2 to 30).")) }
                    textarea id="options" name="options" rows="5" required aria-describedby="options-desc" class=(INPUT)
                        placeholder=(l.t("Pizzeria\nJaponais\nCrêperie", "Pizza place\nSushi bar\nTaco truck")) {}
                }
                (fieldset(l.t("Qui peut voter ?", "Who can vote?"), html! {
                    (radio("mode", "open", true, l.t("Toute personne ayant le lien", "Anyone with the link"),
                        l.t("Un seul lien à partager. Un nom ne peut voter qu'une fois et le navigateur est reconnu. Pratique, mais contournable par quelqu'un de mal intentionné.",
                            "One link to share. Each name votes once and the browser is remembered. Convenient, but a malicious person can get around it.")))
                    (radio("mode", "invite", false, l.t("Sur invitation", "Invitation only"),
                        l.t("Vous saisissez les votants, chacun reçoit un lien personnel valable pour un seul bulletin. Fiable contre le double vote.",
                            "You list the voters, each gets a personal link valid for one ballot. Reliable against double voting.")))
                    div class="hidden pl-7 group-has-[#mode-invite:checked]/form:block" {
                        label for="voters" class=(LABEL) { (l.t("Votants", "Voters")) }
                        p id="voters-desc" class={"mt-1 " (MUTED)} { (l.t("Un nom par ligne. Les liens seront affichés sur la page d'administration.", "One name per line. Links will be shown on the admin page.")) }
                        textarea id="voters" name="voters" rows="4" aria-describedby="voters-desc" class=(INPUT) {}
                    }
                }))
                (fieldset(l.t("Quand afficher les résultats ?", "When are results shown?"), html! {
                    (radio("results", "closed", true, l.t("À la clôture", "When the poll closes"),
                        l.t("Recommandé pour un vote secret.", "Recommended for a secret ballot.")))
                    (radio("results", "live", false, l.t("En direct", "Live"),
                        l.t("Visibles dès 3 bulletins. Attention : en comparant les résultats avant et après le vote de quelqu'un, on peut deviner son bulletin.",
                            "Visible from 3 ballots. Beware: comparing results before and after someone votes can reveal their ballot.")))
                }))
                (fieldset(l.t("Bulletins", "Ballots"), html! {
                    (radio("ballots", "editable", true, l.t("Modifiables", "Editable"),
                        l.t("Chacun peut changer son vote jusqu'à la clôture. Un lien technique caché relie le bulletin au votant (personne ne le voit dans l'appli).",
                            "Voters can change their ballot until the poll closes. A hidden technical link ties the ballot to the voter (nobody sees it in the app).")))
                    (radio("ballots", "secret", false, l.t("Secret strict", "Strictly secret"),
                        l.t("Bulletin définitif, sans aucun lien avec le votant, même dans la base de données.",
                            "Final ballot, with no link to the voter at all, even in the database.")))
                }))
                div class="flex justify-end border-t border-gray-900/10 pt-6" {
                    button type="submit" class=(BTN) { (l.t("Créer le vote", "Create poll")) }
                }
            }
        },
    )
}

fn copy_field(l: Lang, id: &str, label: &str, url: &str) -> Markup {
    html! {
        div {
            label for=(id) class=(LABEL) { (label) }
            div class="mt-2 flex gap-2" {
                input id=(id) type="text" readonly value=(url) onfocus="this.select()"
                    class="block w-full min-w-0 rounded-md bg-gray-50 px-3 py-1.5 font-mono text-sm text-gray-900 outline-1 -outline-offset-1 outline-gray-300 focus:outline-2 focus:-outline-offset-2 focus:outline-indigo-600";
                (copy_button(l, id, label))
            }
        }
    }
}

fn copy_button(l: Lang, target: &str, what: &str) -> Markup {
    let done = l.t("Copié ✓", "Copied ✓");
    html! {
        button type="button" class=(BTN2) aria-label={(l.t("Copier : ", "Copy: ")) (what)}
            onclick={"navigator.clipboard.writeText(document.getElementById('" (target) "').value);var b=this,t=b.textContent;b.textContent='" (done) "';setTimeout(function(){b.textContent=t},1500)"} {
            (l.t("Copier", "Copy"))
        }
    }
}

fn status_badges(l: Lang, p: &Poll, count: usize) -> Markup {
    html! {
        div class="mt-3 flex flex-wrap gap-2" {
            @if p.closed { (badge("gray", l.t("Clos", "Closed"))) } @else { (badge("green", l.t("Ouvert", "Open"))) }
            (badge("indigo", &format!("{count} {}", if count > 1 || (l == Lang::En && count == 0) { l.t("bulletins", "ballots") } else { l.t("bulletin", "ballot") })))
            @if p.invite { (badge("gray", l.t("Sur invitation", "Invitation only"))) }
            @if !p.editable { (badge("gray", l.t("Secret strict", "Strictly secret"))) }
            @if p.live { (badge("gray", l.t("Résultats en direct", "Live results"))) }
        }
    }
}

fn icon(path: &str) -> Markup {
    html! { svg viewBox="0 0 20 20" fill="currentColor" aria-hidden="true" class="size-5" { path fill-rule="evenodd" clip-rule="evenodd" d=(path) {} } }
}

const ICON_UP: &str = "M10 17a.75.75 0 0 1-.75-.75V5.612L5.29 9.77a.75.75 0 0 1-1.08-1.04l5.25-5.5a.75.75 0 0 1 1.08 0l5.25 5.5a.75.75 0 1 1-1.08 1.04l-3.96-4.158V16.25A.75.75 0 0 1 10 17Z";
const ICON_DOWN: &str = "M10 3a.75.75 0 0 1 .75.75v10.638l3.96-4.158a.75.75 0 1 1 1.08 1.04l-5.25 5.5a.75.75 0 0 1-1.08 0l-5.25-5.5a.75.75 0 1 1 1.08-1.04l3.96 4.158V3.75A.75.75 0 0 1 10 3Z";
const ICON_GRIP: &str = "M7 4a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Zm0 6a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Zm-1.5 7.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3ZM16 4a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Zm-1.5 7.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3Zm1.5 4.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Z";

const RANKING_JS: &str = r#"
(function () {
  var list = document.getElementById('ranking-list');
  var out = document.getElementById('ranking');
  var live = document.getElementById('ranking-live');
  var items = function () { return Array.prototype.slice.call(list.children); };
  var tieBtn = function (li) { return li.querySelector('.tie'); };
  var tied = function (li) { return tieBtn(li).getAttribute('aria-pressed') === 'true'; };
  var setTie = function (li, v) { tieBtn(li).setAttribute('aria-pressed', v ? 'true' : 'false'); };
  var init = list.dataset.ranks ? list.dataset.ranks.split(',').map(Number) : null;
  var arr = items();
  if (init) {
    arr.sort(function (a, b) { return init[a.dataset.i] - init[b.dataset.i]; });
  } else {
    // Random initial order, so nobody is favored by the order options were typed in.
    for (var k = arr.length - 1; k > 0; k--) { var j = Math.floor(Math.random() * (k + 1)); var x = arr[k]; arr[k] = arr[j]; arr[j] = x; }
  }
  arr.forEach(function (li, k) { list.appendChild(li); if (init && k) setTie(li, init[li.dataset.i] === init[arr[k - 1].dataset.i]); });

  function refresh() {
    var ranks = [], pos = 0, all = items();
    all.forEach(function (li, k) {
      if (k === 0) setTie(li, false);
      if (!tied(li)) pos = k + 1;
      li.querySelector('.pos').textContent = pos;
      li.querySelector('.up').disabled = k === 0;
      li.querySelector('.down').disabled = k === all.length - 1;
      tieBtn(li).disabled = k === 0;
      ranks[li.dataset.i] = pos;
    });
    out.value = ranks.join(',');
  }
  function say(li) {
    live.textContent = li.dataset.name + ' : ' + list.dataset.posLabel + ' ' + li.querySelector('.pos').textContent + (tied(li) ? ' (' + list.dataset.tieLabel + ')' : '');
  }
  list.addEventListener('click', function (e) {
    var b = e.target.closest('button'); if (!b) return;
    var li = b.closest('li');
    if (b.classList.contains('tie')) setTie(li, !tied(li));
    if (b.classList.contains('up') && li.previousElementSibling) { setTie(li, false); list.insertBefore(li, li.previousElementSibling); }
    if (b.classList.contains('down') && li.nextElementSibling) { setTie(li, false); list.insertBefore(li.nextElementSibling, li); }
    refresh();
    (b.disabled ? li.querySelector(b.classList.contains('up') ? '.down' : '.up') : b).focus();
    say(li);
  });
  Sortable.create(list, { handle: '.handle', animation: 150, ghostClass: 'opacity-40',
    onEnd: function (e) { setTie(e.item, false); refresh(); say(e.item); } });
  refresh();
  list.closest('form').addEventListener('submit', refresh);
})();
"#;

fn ranking_form(l: Lang, p: &Poll, voter: Option<&Voter>, my_ranks: Option<String>) -> Markup {
    html! {
        form method="post" action={"/p/" (p.id) "/vote"} class="space-y-6" {
            @match voter {
                Some(v) => {
                    @if p.invite { input type="hidden" name="t" value=(v.token); }
                    p class=(MUTED) { (l.t("Vous votez en tant que ", "You are voting as ")) strong class="text-gray-900" { (v.name) } "." }
                }
                None => div {
                    label for="name" class=(LABEL) { (l.t("Votre nom", "Your name")) }
                    input id="name" name="name" type="text" required maxlength="100" autocomplete="name" class={(INPUT) " sm:max-w-xs"};
                }
            }
            div {
                h3 id="ranking-label" class=(LABEL) { (l.t("Votre classement", "Your ranking")) }
                p id="ranking-help" class={"mt-1 " (MUTED)} {
                    (l.t("Glissez les options (poignée ⠿) ou utilisez les flèches : la première est votre préférée. Le bouton « = » met une option ex æquo avec celle du dessus.",
                         "Drag options (⠿ handle) or use the arrows: the top one is your favorite. The “=” button ties an option with the one above."))
                }
                noscript { p class="mt-2 text-sm font-medium text-red-700" { (l.t("JavaScript est nécessaire pour classer les options.", "JavaScript is required to rank options.")) } }
                ol id="ranking-list" role="list" aria-labelledby="ranking-label" aria-describedby="ranking-help"
                    data-ranks=[my_ranks] data-pos-label=(l.t("position", "position")) data-tie-label=(l.t("ex æquo", "tied"))
                    class="mt-4 space-y-2" {
                    @for (i, o) in p.options.iter().enumerate() {
                        li data-i=(i) data-name=(o) class="flex items-center gap-x-2 rounded-lg bg-white p-2 ring-1 ring-gray-200 has-aria-pressed:ml-6 has-aria-pressed:ring-indigo-300" {
                            span class="handle cursor-grab touch-none rounded-md p-2 text-gray-400 hover:text-gray-600 active:cursor-grabbing" aria-hidden="true" { (icon(ICON_GRIP)) }
                            span class="pos flex size-7 shrink-0 items-center justify-center rounded-full bg-indigo-50 text-sm font-semibold text-indigo-700" {}
                            span class="min-w-0 flex-1 text-sm/6 font-medium break-words text-gray-900" { (o) }
                            button type="button" class={"tie min-w-9 font-semibold aria-pressed:bg-indigo-600 aria-pressed:text-white aria-pressed:hover:bg-indigo-500 " (ICON_BTN)}
                                aria-pressed="false" aria-label={(l.t("Ex æquo avec l'option du dessus : ", "Tied with the option above: ")) (o)} { "=" }
                            button type="button" class={"up " (ICON_BTN)} aria-label={(l.t("Monter : ", "Move up: ")) (o)} { (icon(ICON_UP)) }
                            button type="button" class={"down " (ICON_BTN)} aria-label={(l.t("Descendre : ", "Move down: ")) (o)} { (icon(ICON_DOWN)) }
                        }
                    }
                }
                div id="ranking-live" class="sr-only" aria-live="polite" {}
                input type="hidden" id="ranking" name="ranking";
            }
            div class="flex items-center justify-end gap-x-4 border-t border-gray-900/10 pt-6" {
                @if !p.editable { p class="text-xs/5 text-gray-500" { (l.t("Votre bulletin sera définitif.", "Your ballot will be final.")) } }
                button type="submit" class=(BTN) { (l.t("Voter", "Vote")) }
            }
        }
        script src="/static/Sortable.min.js" {}
        script { (PreEscaped(RANKING_JS)) }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn poll(
    l: Lang,
    p: &Poll,
    voter: Option<&Voter>,
    my_ranks: Option<String>,
    participants: &[String],
    count: usize,
    res: Option<&Results>,
    just_voted: bool,
) -> Markup {
    let voted = voter.is_some_and(|v| v.voted);
    page(
        l,
        &p.title,
        html! {
            div {
                h1 class="text-2xl font-bold tracking-tight break-words text-gray-900 sm:text-3xl" { (p.title) }
                (status_badges(l, p, count))
            }
            @if just_voted { (alert("success", html! { (l.t("Merci, votre vote est enregistré.", "Thank you, your vote has been recorded.")) })) }
            @if !p.closed {
                section aria-labelledby="vote-title" class=(CARD) {
                    h2 id="vote-title" class={(H2) " mb-4"} { (l.t("Voter", "Vote")) }
                    @if p.invite && voter.is_none() {
                        p class=(MUTED) { (l.t("Ce vote est sur invitation : utilisez le lien personnel qui vous a été envoyé.", "This poll is invitation-only: please use the personal link you were sent.")) }
                    } @else if voted && !p.editable {
                        p class=(MUTED) { (l.t("Vous avez voté. Ce bulletin est définitif.", "You have voted. This ballot is final.")) }
                    } @else {
                        @if voted { div class="mb-6" { (alert("info", html! { (l.t("Vous avez déjà voté. Vous pouvez modifier votre classement jusqu'à la clôture.", "You have already voted. You can change your ranking until the poll closes.")) })) } }
                        (ranking_form(l, p, voter, my_ranks))
                    }
                }
            }
            (results_section(l, p, res, count))
            section aria-labelledby="voters-title" class=(CARD) {
                h2 id="voters-title" class=(H2) { (l.t("Ont voté", "Voted")) " (" (participants.len()) ")" }
                @if participants.is_empty() {
                    p class={"mt-2 " (MUTED)} { (l.t("Personne pour l'instant.", "Nobody yet.")) }
                } @else {
                    ul role="list" class="mt-3 flex flex-wrap gap-2" {
                        @for n in participants { li class="rounded-full bg-gray-100 px-3 py-1 text-sm text-gray-700" { (n) } }
                    }
                }
            }
            (verification(l, p))
        },
    )
}

fn verification(l: Lang, p: &Poll) -> Markup {
    html! {
        details class={"group " (CARD)} {
            summary class="cursor-pointer rounded-md text-sm font-semibold text-gray-900 focus-visible:outline-2 focus-visible:outline-indigo-600" {
                (l.t("Comment le gagnant est-il désigné ?", "How is the winner chosen?"))
            }
            div class={"mt-4 space-y-3 " (MUTED)} {
                p { (l.t(
                    "Pour chaque paire d'options, on compte combien de votants préfèrent l'une à l'autre. Si une option gagne tous ses duels, elle gagne. Sinon (cycle du type A > B > C > A), on calcule la loterie optimale : la répartition de probabilités qui, en moyenne, ne perd contre aucune option. Le gagnant est tiré au sort selon cette loterie.",
                    "For each pair of options, we count how many voters prefer one to the other. If an option wins all its duels, it wins. Otherwise (a cycle like A > B > C > A), we compute the optimal lottery: the probability split that, on average, loses to no option. The winner is drawn from that lottery."
                )) " " a class=(LINK) href=(VIDEO) { (l.t("Voir la vidéo", "Watch the video")) } }
                p { (l.t(
                    "Tirage vérifiable : un nombre secret a été tiré à la création du vote. Son empreinte SHA-256 est publiée ci-dessous dès maintenant ; le nombre est révélé à la clôture, et le tirage en découle. Personne ne peut donc relancer le tirage.",
                    "Verifiable draw: a secret number was drawn when the poll was created. Its SHA-256 fingerprint is published below right away; the number is revealed at closing, and the draw follows from it. So nobody can re-roll the draw."
                )) }
                p { (l.t("Empreinte : ", "Fingerprint: ")) code class="font-mono text-xs break-all text-gray-900" { (commitment(&p.seed)) } }
            }
        }
    }
}

fn pct(p: f64) -> String {
    format!("{:.1}", p * 100.0)
}

fn results_section(l: Lang, p: &Poll, res: Option<&Results>, count: usize) -> Markup {
    html! {
        section aria-labelledby="results-title" class=(CARD) {
            h2 id="results-title" class=(H2) {
                @if p.closed { (l.t("Résultats", "Results")) } @else { (l.t("Résultats provisoires", "Provisional results")) }
            }
            @match res {
                None => p class={"mt-2 " (MUTED)} {
                    @if p.live {
                        (l.t("Les résultats s'afficheront à partir de ", "Results will show from ")) (MIN_LIVE_BALLOTS)
                        (l.t(" bulletins, pour préserver le secret du vote.", " ballots, to protect ballot secrecy."))
                        " (" (count) ")"
                    } @else {
                        (l.t("Les résultats seront visibles à la clôture du vote.", "Results will be visible once the poll is closed."))
                    }
                },
                Some(r) if r.ballots == 0 => p class={"mt-2 " (MUTED)} { (l.t("Aucun bulletin.", "No ballots.")) },
                Some(r) => (results_body(l, p, r)),
            }
        }
    }
}

fn results_body(l: Lang, p: &Poll, r: &Results) -> Markup {
    let o = &p.options;
    let n = o.len();
    let wins = |i: usize| (0..n).filter(|&j| r.duels[i][j] > r.duels[j][i]).count();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| r.probs[b].total_cmp(&r.probs[a]).then(wins(b).cmp(&wins(a))));
    html! {
        div class="mt-4 space-y-8" {
            @if let Some((_, _, w)) = &r.draw {
                div class="rounded-lg bg-indigo-600 p-5 text-white" {
                    p class="text-sm/6 text-indigo-100" { (l.t("Gagnant", "Winner")) }
                    p class="text-2xl font-bold break-words" { "🏆 " (o[*w]) }
                    p class="mt-1 text-sm/6 text-indigo-100" {
                        @if r.winner.is_some() { (l.t("Gagnant de Condorcet : bat toutes les autres options en duel.", "Condorcet winner: beats every other option head-to-head.")) }
                        @else { (l.t("Pas de gagnant de Condorcet : désigné par tirage au sort selon la loterie optimale.", "No Condorcet winner: drawn from the optimal lottery.")) }
                    }
                }
            } @else {
                @match r.winner {
                    Some(w) => (alert("info", html! { strong { (o[w]) } (l.t(" bat actuellement toutes les autres options en duel.", " currently beats every other option head-to-head.")) })),
                    None => (alert("warning", html! { (l.t("Pas de gagnant de Condorcet pour l'instant : un tirage au sort selon la loterie ci-dessous aura lieu à la clôture.", "No Condorcet winner so far: a draw from the lottery below will happen at closing.")) })),
                }
            }

            div {
                h3 class=(LABEL) { (l.t("Loterie optimale", "Optimal lottery")) }
                ul role="list" class="mt-3 space-y-3" {
                    @for &i in &order {
                        li {
                            div class="flex justify-between gap-4 text-sm/6" {
                                span class="min-w-0 break-words text-gray-900" { (o[i]) }
                                span class="font-semibold tabular-nums text-gray-900" { (pct(r.probs[i])) " %" }
                            }
                            div class="mt-1 h-2 overflow-hidden rounded-full bg-gray-100" aria-hidden="true" {
                                div class="h-2 rounded-full bg-indigo-600" style={"width:" (pct(r.probs[i])) "%"} {}
                            }
                        }
                    }
                }
            }

            div {
                h3 class=(LABEL) { (l.t("Duels", "Head-to-head")) }
                div class="mt-3 -mx-6 overflow-x-auto px-6 sm:-mx-8 sm:px-8" {
                    table class="min-w-full text-sm" {
                        caption class={"mb-2 text-left " (MUTED)} {
                            (l.t("Chaque case indique combien de votants préfèrent l'option de la ligne à celle de la colonne. Vert : la ligne gagne le duel.", "Each cell shows how many voters prefer the row option to the column option. Green: the row wins the duel."))
                        }
                        thead {
                            tr {
                                th scope="col" { span class="sr-only" { (l.t("Option", "Option")) } }
                                @for &j in &order { th scope="col" class="max-w-28 px-2 py-2 text-left align-bottom font-medium break-words text-gray-700" { (o[j]) } }
                            }
                        }
                        tbody class="divide-y divide-gray-100" {
                            @for &i in &order {
                                tr {
                                    th scope="row" class="max-w-40 py-2 pr-3 text-left font-medium break-words text-gray-900" { (o[i]) }
                                    @for &j in &order {
                                        @if i == j {
                                            td class="px-2 py-2 text-center text-gray-300" { "—" }
                                        } @else {
                                            @let (a, b) = (r.duels[i][j], r.duels[j][i]);
                                            td class={"px-2 py-2 text-center tabular-nums " (if a > b { "bg-green-50 font-semibold text-green-800" } else if a < b { "bg-red-50 text-red-800" } else { "text-gray-600" })} { (a) }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            @if let (Some((seed, u, w)), None) = (&r.draw, r.winner) {
                div class={"space-y-1 " (MUTED)} {
                    h3 class=(LABEL) { (l.t("Vérification du tirage", "Draw verification")) }
                    p { (l.t("Nombre secret révélé : ", "Revealed secret: ")) code class="font-mono text-xs break-all text-gray-900" { (seed) } }
                    p { (l.t("Son empreinte SHA-256 doit être égale à l'empreinte publiée : ", "Its SHA-256 must equal the published fingerprint: ")) code class="font-mono text-xs break-all text-gray-900" { (commitment(&p.seed)) } }
                    p {
                        (l.t("Tirage u = 8 premiers octets / 2⁶⁴ = ", "Draw u = first 8 bytes / 2⁶⁴ = ")) span class="font-mono text-gray-900" { (format!("{u:.6}")) }
                        (l.t(", qui tombe dans l'intervalle de ", ", which falls in the interval of ")) strong class="text-gray-900" { (o[*w]) }
                        (l.t(" (options cumulées dans l'ordre de saisie).", " (options accumulated in input order)."))
                    }
                }
            }
        }
    }
}

pub fn admin(
    l: Lang,
    base: &str,
    p: &Poll,
    voters: &[Voter],
    count: usize,
    res: Option<&Results>,
) -> Markup {
    let poll_url = format!("{base}/p/{}", p.id);
    page(
        l,
        &p.title,
        html! {
            div {
                p class="text-sm font-semibold text-indigo-600" { (l.t("Administration", "Admin")) }
                h1 class="mt-1 text-2xl font-bold tracking-tight break-words text-gray-900 sm:text-3xl" { (p.title) }
                (status_badges(l, p, count))
            }
            (alert("warning", html! {
                strong { (l.t("Gardez cette page secrète et ajoutez-la à vos favoris. ", "Keep this page secret and bookmark it. ")) }
                (l.t("Son adresse est la seule clé d'administration : il n'y a pas de compte.", "Its address is the only admin key: there is no account."))
            }))
            section aria-labelledby="links-title" class={"space-y-6 " (CARD)} {
                h2 id="links-title" class=(H2) { (l.t("Liens", "Links")) }
                (copy_field(l, "admin-url", l.t("Lien d'administration (pour vous)", "Admin link (for you)"), &format!("{base}/p/{}/admin/{}", p.id, p.admin_key)))
                @if !p.invite {
                    (copy_field(l, "poll-url", l.t("Lien de vote (à partager)", "Voting link (to share)"), &poll_url))
                    p { a class=(LINK) href={"/p/" (p.id)} { (l.t("Ouvrir la page de vote →", "Open the voting page →")) } }
                } @else {
                    div {
                        h3 class=(LABEL) { (l.t("Liens personnels (un par votant, à envoyer individuellement)", "Personal links (one per voter, send individually)")) }
                        ul role="list" class="mt-3 divide-y divide-gray-100" {
                            @for (k, v) in voters.iter().enumerate() {
                                @let id = format!("voter-{k}");
                                li class="flex flex-wrap items-center gap-x-4 gap-y-2 py-3" {
                                    div class="min-w-0 flex-1" {
                                        p class="text-sm/6 font-semibold break-words text-gray-900" { (v.name) }
                                        input id=(id) type="text" readonly value={(poll_url) "?t=" (v.token)} onfocus="this.select()"
                                            aria-label={(l.t("Lien de ", "Link for ")) (v.name)}
                                            class="mt-1 block w-full truncate rounded-md bg-gray-50 px-2 py-1 font-mono text-xs text-gray-600 outline-1 -outline-offset-1 outline-gray-200 focus:outline-2 focus:outline-indigo-600";
                                    }
                                    @if v.voted { (badge("green", l.t("A voté", "Voted"))) } @else { (badge("gray", l.t("En attente", "Pending"))) }
                                    (copy_button(l, &id, &v.name))
                                }
                            }
                        }
                    }
                }
            }
            @if !p.closed {
                section aria-labelledby="close-title" class=(CARD) {
                    h2 id="close-title" class=(H2) { (l.t("Clôturer le vote", "Close the poll")) }
                    p class={"mt-2 " (MUTED)} { (l.t(
                        "La clôture est définitive : plus personne ne peut voter, les résultats deviennent visibles et, s'il n'y a pas de gagnant de Condorcet, le tirage au sort a lieu.",
                        "Closing is final: nobody can vote anymore, results become visible and, if there is no Condorcet winner, the draw takes place."
                    )) }
                    form method="post" action={"/p/" (p.id) "/admin/" (p.admin_key) "/close"} class="mt-4"
                        onsubmit={"return confirm('" (l.t("Clôturer définitivement ce vote ?", "Close this poll for good?")) "')"} {
                        button type="submit" class=(BTN_DANGER) { (l.t("Clôturer et désigner le gagnant", "Close and pick the winner")) }
                    }
                }
            }
            (results_section(l, p, res, count))
        },
    )
}
