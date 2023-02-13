use crate::{components::layout::Layout, DEFAULT_IMAGE_ENDPOINT, SHORTS_ENDPOINT, CATAGORIES_ENDPOINT};
use perseus::{prelude::*, state::rx_collections::RxVec};
use sycamore::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(ReactiveState, Serialize, Deserialize, Clone)]
#[rx(alias = "IndexPageStateRx")]
pub struct IndexPageState {
    pub catagories: Vec<Catagory>,
    #[rx(nested)]
    recipes: RxVec<Short>,
    filters: String,
    catagory_filter: String,
    default_img: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Eq)]
pub struct Catagory {
    name: String,
}

#[auto_scope]
pub fn index_page<G: Html>(cx: Scope, props: &IndexPageStateRx) -> View<G> {
    view! { cx,
        Layout(title = "test")
        {
        div(class = "search-bar") {
            input(
                placeholder = "Enter Search",
                bind:value = props.filters
            )
        }
        // a(href= format!("recipe/{}", name), style = "display:block; text-decoration:none;") {
        div
        {
            Indexed(
                iterable = &props.recipes,
                view = move |cx, short| view! { cx,
                (if short.get().recipe.name.to_lowercase().contains(&*props.filters.get().to_lowercase())
                {
                    let image_loc = short.get().image.clone();
                    let recipe_name = short.get().recipe.name.clone();
                    let recipe_link = format!("recipe/{}", recipe_name.clone());
                    let recipe_base_amount = short.get().recipe.base_amount.clone();
                    let recipe_unit_name = short.get().recipe.unit_name.clone();
                    let ingredients = create_signal(cx, short.get().ingredients.clone());
                    view! {cx,
                        a(class = "recipe_link", href = format!("recipe/{}", recipe_link)){
                        table(class = "short-list")
                        {
                            tr
                            {
                                th(class = "short-image", rowspan = 2)
                                {
                                    (if let Some(ref image_location) = &image_loc
                                    {
                                        let location_clone = image_location.clone();
                                        view! { cx, td { img(src=location_clone)}}
                                    }
                                    else
                                    {
                                        view! { cx, td { img(src=&props.default_img.get())}}
                                    })
                                }
                                th
                                {
                                    p { (recipe_name)}
                                    p { (format!("{} {}", recipe_base_amount, recipe_unit_name))}
                                }
                            }
                            tr
                            {
                                td
                                {
                                    ul
                                    {
                                        Indexed
                                        (
                                            iterable = ingredients,
                                            view = |cx, ingredient| 
                                            view! {cx, 
                                                li { (format!("{} {} {}", ingredient.amount, ingredient.unit, ingredient.name)) } 
                                            }
                                        )
                                    }
                                }
                            }
                        } 
                    }
                    }
                }
                else
                {
                    view! { cx, }
                }
                )}
            )
        }
    }
    }
}

#[engine_only_fn]
pub fn head(cx: Scope) -> View<SsrNode> {
    view! { cx,
        title { "Heuts" }
    }
}

pub fn get_template<G: Html>() -> Template<G> {
    Template::build("index")
        .build_paths_fn(get_build_paths)
        .build_state_fn(get_build_state)
        .revalidate_after("30s")
        .view_with_state(index_page)
        .incremental_generation()
        .build()
}

#[engine_only_fn]
async fn get_build_state(
    info: StateGeneratorInfo<HelperState>,
) -> Result<IndexPageState, BlamedError<reqwest::Error>> {
    let resp = if info.path == "" {
        reqwest::get(&*SHORTS_ENDPOINT)
            .await?
            .text()
            .await?
    } else {
        reqwest::get(format!(
            "{}/catagory/{}",
            &*SHORTS_ENDPOINT,
            info.path
        ))
        .await?
        .text()
        .await?
    };
    let recipes = serde_json::from_str(&resp).unwrap();
    Ok(IndexPageState {
        catagories: info.get_extra().catagories,
        recipes,
        filters: "".to_string(),
        catagory_filter: info.path,
        default_img: (DEFAULT_IMAGE_ENDPOINT).to_string(),
    })
}

#[derive(Serialize, Deserialize)]
struct HelperState {
    catagories: Vec<Catagory>,
}

#[engine_only_fn]
async fn get_build_paths() -> BuildPaths {
    let resp = reqwest::get(&*CATAGORIES_ENDPOINT)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let catagories: Vec<Catagory> = serde_json::from_str(&resp).unwrap();
    let paths = BuildPaths {
        paths: catagories.clone().into_iter().map(|x| x.name).collect(),
        extra: HelperState { catagories }.into(),
    };
    paths
}

#[derive(Serialize, Deserialize, Clone, PartialEq, UnreactiveState)]
pub struct Short {
    recipe: Recipe,
    ingredients: Vec<IngredientAndAmount>,
    image: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct IngredientAndAmount {
    name: String,
    amount: f32,
    unit: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Recipe {
    name: String,
    catagory_name: String,
    information: Option<String>,
    base_amount: f32,
    unit_name: String,
    preparation: Option<String>,
}
