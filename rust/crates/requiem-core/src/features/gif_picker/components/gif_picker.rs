use crate::ui::classes::emoji_picker as classes_emoji_picker;
use crate::ui::classes::gif_picker as cls;
use crate::ui::emoji_picker;
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::StackProps;
use requiem_ui::icons::Icon;
use requiem_ui::signal::signal;
use skia_safe::font_style::Weight;
use std::rc::Rc;

use requiem_ui::components::{ButtonProps, ScrollViewProps, TextProps, div, icon, text};
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use requiem_ui::widget::primitives::ImageWidget;

use crate::images::ImageCache;
use crate::rest::endpoints::favorite_gifs::{FavoriteGif, FavoriteGifs, GIF_TYPE_IMAGE};
use crate::rest::endpoints::gifs::{GifCategory, GifResult};

use crate::ui::{TEXT_DEFAULT, TEXT_STRONG};
use requiem_ui::icons::{ANALYTICS, ARROW_SMALL_LEFT, STAR, STAR_OUTLINE};

const EMPTY_HINTS: [&str; 5] = [
    requiem_i18n::key!("gif-picker-click-the-star-in-the-corner"),
    requiem_i18n::key!("gif-picker-favorites-will-show-up-here"),
    requiem_i18n::key!("gif-picker-so-uhh-maybe-go-favorite-some"),
    requiem_i18n::key!("gif-picker-uh-go-favorite-them-please"),
    requiem_i18n::key!("gif-picker-why-uh-are-you-still-here"),
];
thread_local! {
    static GIF_SCROLL: Signal<f32> = signal(0.0);
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum GifView {
    #[default]
    Browse,
    Favorites,
    Trending,
}

#[derive(Clone)]
pub struct GifPickerSignals {
    pub search: Signal<String>,

    pub categories: Signal<Vec<GifCategory>>,

    pub trending: Signal<Vec<GifResult>>,

    pub results: Signal<Vec<GifResult>>,

    pub favorites: Signal<FavoriteGifs>,
    pub view: Signal<GifView>,

    pub image_tick: Signal<u64>,
}

impl GifPickerSignals {
    pub fn inert() -> Self {
        Self {
            search: signal(String::new()),
            categories: signal(Vec::new()),
            trending: signal(Vec::new()),
            results: signal(Vec::new()),
            favorites: signal(FavoriteGifs::default()),
            view: signal(GifView::Browse),
            image_tick: signal(0),
        }
    }
}

#[derive(Clone)]
pub struct GifPickerState {
    pub signals: GifPickerSignals,

    pub on_search: Rc<dyn Fn(String)>,

    pub on_open: Rc<dyn Fn()>,

    pub on_save_favorites: Rc<dyn Fn(FavoriteGifs)>,
}

impl GifPickerState {
    pub fn inert() -> Self {
        Self {
            signals: GifPickerSignals::inert(),
            on_search: Rc::new(|_| {}),
            on_open: Rc::new(|| {}),
            on_save_favorites: Rc::new(|_| {}),
        }
    }
}

#[derive(Clone)]
struct GifTile {
    url: String,
    thumbnail: String,
    favorite: FavoriteGif,
}

impl GifTile {
    fn from_result(result: &GifResult) -> Self {
        Self {
            url: result.url.clone(),
            thumbnail: result.preview.clone(),
            favorite: FavoriteGif {
                url: result.url.clone(),
                format: GIF_TYPE_IMAGE,
                src: result.gif_src.clone(),
                width: result.width,
                height: result.height,
                order: 0,
            },
        }
    }

    fn from_favorite(favorite: &FavoriteGif) -> Self {
        Self {
            url: favorite.url.clone(),
            thumbnail: favorite.src.clone(),
            favorite: favorite.clone(),
        }
    }
}

pub fn build(state: &GifPickerState, images: &ImageCache, on_pick: Rc<dyn Fn(String)>) -> Node {
    let search = state.signals.search.get();
    let view = state.signals.view.get();
    let favorites = state.signals.favorites.get();
    // A new grid is a new scroll view, which starts at the top.
    GIF_SCROLL.with(|scroll| scroll.set(0.0));

    let searching = !search.trim().is_empty();
    let (header, grid) = if searching {
        let tiles: Vec<GifTile> = state
            .signals
            .results
            .get()
            .iter()
            .map(GifTile::from_result)
            .collect();
        (
            build_search_header(&state.signals.search, &state.on_search, &search),
            result_grid(&tiles, state, &favorites, images, &on_pick),
        )
    } else {
        match view {
            GifView::Browse => (
                build_search_header(&state.signals.search, &state.on_search, &search),
                browse_grid(state, &favorites, images),
            ),
            GifView::Favorites => {
                let tiles: Vec<GifTile> = favorites
                    .sorted()
                    .iter()
                    .map(GifTile::from_favorite)
                    .collect();
                let grid = if tiles.is_empty() {
                    empty_hints()
                } else {
                    result_grid(&tiles, state, &favorites, images, &on_pick)
                };
                (
                    subview_header(
                        &requiem_i18n::t!("gif-picker-favorites"),
                        &state.signals.view,
                    ),
                    grid,
                )
            }
            GifView::Trending => {
                let tiles: Vec<GifTile> = state
                    .signals
                    .trending
                    .get()
                    .iter()
                    .map(GifTile::from_result)
                    .collect();
                (
                    subview_header(
                        &requiem_i18n::t!("gif-picker-trending-gifs"),
                        &state.signals.view,
                    ),
                    result_grid(&tiles, state, &favorites, images, &on_pick),
                )
            }
        }
    };

    let body = ScrollViewProps::default()
        .on_scroll(move |offset: f32, _max: f32| {
            let seen = GIF_SCROLL.with(Signal::peek);
            if (offset - seen).abs() > 152.0 {
                GIF_SCROLL.with(|scroll| scroll.set(offset));
            }
        })
        .build()
        .class(cls::GIF_BODY_SCROLL)
        .child(grid);
    div().class(cls::GIF_TAB).child(header).child(body)
}

fn build_search_header(
    search_signal: &Signal<String>,
    on_search: &Rc<dyn Fn(String)>,
    search: &str,
) -> Node {
    let search_signal = search_signal.clone();
    let on_search = on_search.clone();
    emoji_picker::search_header(
        &requiem_i18n::t!("gif-picker-search-klipy"),
        search,
        move |text: &str| {
            search_signal.set(text.to_string());
            on_search(text.to_string());
        },
        None,
        None,
        false,
    )
}

fn subview_header(title: &str, view: &Signal<GifView>) -> Node {
    let view = view.clone();
    let back = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::GIF_BACK_BTN.to_string()])
        .on_click(move || view.set(GifView::Browse))
        .build()
        .child(icon(ARROW_SMALL_LEFT, TEXT_DEFAULT).class(cls::GIF_BACK_ICON));
    let title = text(TextProps {
        content: title.to_string(),
        size: 16.0,
        color: TEXT_STRONG,
        weight: Weight::SEMI_BOLD.into(),
        ..Default::default()
    })
    .class(cls::GIF_SUBVIEW_TITLE);
    div()
        .class(classes_emoji_picker::PICKER_HEADER)
        .class(cls::GIF_SUBVIEW_HEADER)
        .child(back)
        .child(title)
}

/// Two tiles a row, and only the rows near the viewport are built. The window is its own
/// reactive node, so scrolling rebuilds the rows and nothing around them (the search field,
/// the scroll view itself).
fn rows_of_two(count: usize, make: Rc<dyn Fn(usize) -> Node>) -> Node {
    const ROW: f32 = 152.0;
    const OVERSCAN: f32 = 304.0;
    let rows_total = count.div_ceil(2);
    Node::reactive(move || {
        let offset = GIF_SCROLL.with(Signal::get);
        let start = ((offset - OVERSCAN) / ROW).floor().max(0.0) as usize;
        let end = (((offset + 486.0 + OVERSCAN) / ROW).ceil() as usize).clamp(start, rows_total);
        let mut grid = div().class(cls::GIF_GRID);
        if start > 0 {
            grid = grid
                .child(div().style(format!("height: {}px; flex-shrink: 0", start as f32 * ROW)));
        }
        for row_index in start..end {
            let first = row_index * 2;
            let mut row = div().class(cls::GIF_ROW).child(make(first));
            if first + 1 < count {
                row = row.child(make(first + 1));
            }
            grid = grid.child(row);
        }
        if end < rows_total {
            grid = grid.child(div().style(format!(
                "height: {}px; flex-shrink: 0",
                (rows_total - end) as f32 * ROW
            )));
        }
        grid
    })
    .style("width: 100%; flex-shrink: 0")
}

fn browse_grid(state: &GifPickerState, favorites: &FavoriteGifs, images: &ImageCache) -> Node {
    enum Tile {
        Favorites(Option<String>),
        Trending(Option<String>),
        Category { name: String, src: String },
    }
    let mut tiles = vec![
        Tile::Favorites(favorites.sorted().first().map(|f| f.src.clone())),
        Tile::Trending(
            state
                .signals
                .trending
                .get()
                .first()
                .map(|g| g.preview.clone()),
        ),
    ];
    tiles.extend(
        state
            .signals
            .categories
            .get()
            .iter()
            .map(|category| Tile::Category {
                name: category.name.clone(),
                src: category.src.clone(),
            }),
    );
    let count = tiles.len();
    let (state, images) = (state.clone(), images.clone());
    rows_of_two(
        count,
        Rc::new(move |index| match &tiles[index] {
            Tile::Favorites(thumb) => {
                let view = state.signals.view.clone();
                category_tile(
                    &requiem_i18n::t!("gif-picker-favorites"),
                    None,
                    thumb.as_deref(),
                    true,
                    &images,
                    &state.signals.image_tick,
                    move || view.set(GifView::Favorites),
                )
            }
            Tile::Trending(thumb) => {
                let view = state.signals.view.clone();
                category_tile(
                    &requiem_i18n::t!("gif-picker-trending-gifs"),
                    Some(ANALYTICS),
                    thumb.as_deref(),
                    false,
                    &images,
                    &state.signals.image_tick,
                    move || view.set(GifView::Trending),
                )
            }
            Tile::Category { name, src } => {
                let (name, search_signal, on_search) = (
                    name.clone(),
                    state.signals.search.clone(),
                    state.on_search.clone(),
                );
                category_tile(
                    &name.clone(),
                    None,
                    Some(src),
                    false,
                    &images,
                    &state.signals.image_tick,
                    move || {
                        search_signal.set(name.clone());
                        on_search(name.clone());
                    },
                )
            }
        }),
    )
}

fn thumbnail_image(src: &str, images: &ImageCache, image_tick: &Signal<u64>) -> Node {
    let initial = images.get_or_decode_image(src);
    let node = Node::new("div", ImageWidget::new(initial.clone()));
    if initial.is_some() {
        return node;
    }
    let images = images.clone();
    let src = src.to_string();
    node.bind_image(image_tick.clone(), move || images.get_or_decode_image(&src))
}

fn category_tile(
    label: &str,
    icon_path: Option<Icon>,
    thumbnail: Option<&str>,
    blurple: bool,
    images: &ImageCache,
    image_tick: &Signal<u64>,
    on_click: impl FnMut() + 'static,
) -> Node {
    let mut tile = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::GIF_CATEGORY_TILE.to_string()])
        .on_click(on_click)
        .build();
    if let Some(src) = thumbnail {
        tile = tile.child(thumbnail_image(src, images, image_tick).class(cls::GIF_CATEGORY_IMAGE));
    }
    let fade = if blurple {
        cls::GIF_CATEGORY_FADE_BLURPLE
    } else {
        cls::GIF_CATEGORY_FADE
    };
    let mut label_row = div().class(cls::GIF_CATEGORY_TEXT);
    if let Some(path) = icon_path {
        label_row = label_row.child(icon(path, TEXT_STRONG).class(cls::GIF_CATEGORY_ICON));
    }
    label_row = label_row.child(
        text(TextProps {
            content: label.to_string(),
            size: 16.0,
            color: TEXT_STRONG,
            weight: Weight::SEMI_BOLD.into(),
            ..Default::default()
        })
        .class(cls::GIF_CATEGORY_LABEL),
    );
    tile.child(div().class(fade)).child(label_row)
}

fn result_grid(
    tiles: &[GifTile],
    state: &GifPickerState,
    favorites: &FavoriteGifs,
    images: &ImageCache,
    on_pick: &Rc<dyn Fn(String)>,
) -> Node {
    let count = tiles.len();
    let (tiles, favorites) = (Rc::new(tiles.to_vec()), favorites.clone());
    let (state, images, on_pick) = (state.clone(), images.clone(), on_pick.clone());
    rows_of_two(
        count,
        Rc::new(move |index| {
            let tile = tiles[index].clone();
            let is_favorite = favorites.contains(&tile.url);
            result_tile(tile, &state, is_favorite, &images, on_pick.clone())
        }),
    )
}

fn result_tile(
    tile: GifTile,
    state: &GifPickerState,
    is_favorite: bool,
    images: &ImageCache,
    on_pick: Rc<dyn Fn(String)>,
) -> Node {
    let url = tile.url.clone();
    let node = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::GIF_RESULT_TILE.to_string()])
        .on_click(move || on_pick(url.clone()))
        .build()
        .child(
            thumbnail_image(&tile.thumbnail, images, &state.signals.image_tick)
                .class(cls::GIF_RESULT_IMAGE),
        );

    let favorites = state.signals.favorites.clone();
    let on_save = state.on_save_favorites.clone();
    let favorite = tile.favorite;
    let mut star = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::GIF_FAV_BTN.to_string()])
        .on_click(move || {
            let mut next = favorites.peek();
            next.toggle(favorite.clone());
            favorites.set(next.clone());
            on_save(next);
        })
        .build()
        .child(
            icon(if is_favorite { STAR } else { STAR_OUTLINE }, TEXT_STRONG)
                .class(cls::GIF_FAV_ICON),
        );
    if is_favorite {
        star = star.class(cls::GIF_FAV_BTN_SELECTED);
    }
    node.child(div().class(cls::GIF_RESULT_HOVER)).child(star)
}

fn empty_hints() -> Node {
    let card = |i: usize, hint: &str| {
        let mut card = div().class(cls::GIF_EMPTY_HINT_CARD).child(
            text(TextProps {
                content: hint.to_string(),
                size: 15.0,
                color: TEXT_DEFAULT,
                center: true,
                ..Default::default()
            })
            .class(cls::GIF_EMPTY_HINT_TEXT),
        );
        if i == 0 {
            card = card.child(icon(STAR, TEXT_STRONG).class(cls::GIF_EMPTY_HINT_STAR));
        }
        div().class(cls::GIF_EMPTY_HINT).child(card)
    };
    let mut hints = div().class(cls::GIF_EMPTY_HINTS);
    let cards: Vec<Node> = EMPTY_HINTS
        .iter()
        .enumerate()
        .map(|(i, h)| card(i, &requiem_i18n::text(h)))
        .collect();
    let mut cards = cards.into_iter();
    loop {
        let mut row = StackProps::horizontal().gap(0).build();
        let mut any = false;
        for _ in 0..3 {
            match cards.next() {
                Some(c) => {
                    row = row.child(c);
                    any = true;
                }
                None => row = row.child(div().class(cls::GIF_EMPTY_HINT)),
            }
        }
        if !any {
            break;
        }
        hints = hints.child(row);
    }
    hints
}
