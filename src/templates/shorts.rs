use crate::{components::layout::Layout, SHORTS_ENDPOINT, CATAGORIES_ENDPOINT, errors::Error, EndpointsRx, common::{IngredientAndAmount, Recipe, Catagory}};
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
                                        let image_endpoint = global_state.image_endpoint.clone(); 
                                        let image = image_location.clone();
                                        view! { cx,
                                            img(src= format!("{}/{}", image_endpoint, image))
                                        }
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
                                                li { (format!("{} {} {}", ingredient.amount, ingredient.unit_name, ingredient.ingredient_name)) } 
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
        .revalidate_after("30s")
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