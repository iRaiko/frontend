use crate::{components::layout::Layout, SHORTS_ENDPOINT, CATAGORIES_ENDPOINT, errors::Error, EndpointsRx};
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
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Eq)]
pub struct Catagory {
    name: String,
}

#[auto_scope]
pub fn index_page<G: Html>(cx: Scope, props: &IndexPageStateRx) -> View<G> {
    let global_state = Reactor::<G>::from_cx(cx).get_global_state::<EndpointsRx>(cx);

    view! { cx,
        Layout(title = "test")
        {
        div(class = "search-bar") {
            input(
                placeholder = "Enter Search",
                bind:value = props.filters
            )
        }
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
                        a(class = "recipe_link", href = recipe_link){
                        table(class = "short-list")
                        {
                            tr
                            {
                                th(class = "short-image", rowspan = "2")
                                {
                                    (if let Some(ref image_location) = &image_loc
                                    {
                                        let location_clone = image_location.clone();
                                        view! { cx, img(src=location_clone)}
                                    }
                                    else
                                    {
                                        view! { cx, img(src= global_state.default_image)}
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

pub fn get_template<G: Html>() -> Template<G> {
    Template::build("index")
        .build_paths_fn(get_build_paths)
        .build_state_fn(get_build_state)
        .revalidate_after("24h")
        .view_with_state(index_page)
        .incremental_generation()
        .build()
}

#[engine_only_fn]
async fn get_build_state(
    info: StateGeneratorInfo<HelperState>,
) -> Result<IndexPageState, BlamedError<Error>> {
    let resp = if info.path == "" {
        reqwest::get(&*SHORTS_ENDPOINT)
            .await.map_err(Error::from)?
            .text()
            .await.map_err(Error::from)?
    } else {
        reqwest::get(format!(
            "{}/catagory/{}",
            &*SHORTS_ENDPOINT,
            info.path
        ))
        .await.map_err(Error::from)?
        .text()
        .await.map_err(Error::from)?
    };
    let recipes = serde_json::from_str(&resp).map_err(Error::from)?;
    Ok(IndexPageState {
        catagories: info.get_extra().catagories,
        recipes,
        filters: "".to_string(),
        catagory_filter: info.path,
    })
}

#[derive(Serialize, Deserialize)]
struct HelperState {
    catagories: Vec<Catagory>,
}

#[engine_only_fn]
async fn get_build_paths() -> Result<BuildPaths, Error> {
    let resp = reqwest::get(&*CATAGORIES_ENDPOINT)
        .await?
        .text()
        .await?;
    let catagories: Vec<Catagory> = serde_json::from_str(&resp)?;
    let paths = BuildPaths {
        paths: catagories.clone().into_iter().map(|x| x.name).collect(),
        extra: HelperState { catagories }.into(),
    };
    Ok(paths)
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
