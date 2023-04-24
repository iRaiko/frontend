use perseus::prelude::*;
use serde::{Deserialize, Serialize};
use sycamore::prelude::*;
use crate::common::{IngredientAndAmount, Recipe};
use crate::components::layout::Layout;
use crate::errors::Error;
use crate::{RECIPES_ENDPOINT, BACKEND};
use crate::EndpointsRx;

// Initialize our app with the `perseus_warp` package's default server (fully
// customizable)
pub fn get_template<G: Html>() -> Template<G> {
    Template::build("recipe")
        .build_paths_fn(get_paths)
        .build_state_fn(get_index_build_state)
        .revalidate_after("30s")
        .view_with_state(recipe_page)
        .incremental_generation()
        .build()
}

// EXCERPT_START
#[auto_scope]
fn recipe_page<G: Html>(cx: Scope, props: &RecipePropsRx) -> View<G> 
{
    let global_state = Reactor::<G>::from_cx(cx).get_global_state::<EndpointsRx>(cx);
    let amount = create_signal(cx, props.recipe.base_amount.get().to_string());
    let a = (*props.ingredients.get()).clone();
    let string_to_f32 = create_memo(cx, || amount.get().parse::<f32>());
    let ingredients = create_signal(
        cx,
        a.into_iter()
            .map(|x| {
                create_memo(cx, move || {
                    let n = if let Ok(number) = *string_to_f32.get() {
                        x.amount * (number / *props.recipe.base_amount.get())
                    } else {
                        x.amount
                    };
                    format!("{} {} {}", n, x.unit_name, x.ingredient_name)
                })
            })
            .collect::<Vec<_>>(),
    );

    let link_to_update_page = format!("update/{}", &props.recipe.name);

    view! { cx,
        Layout(title = "test")
        {
                a(href = link_to_update_page) { "test" }
                h1 { (props.recipe.name)}
                div(class = "full_recipe_images"){
                Indexed(
                    iterable = &props.images,
                    view = |cx, x|
                    {
                        let image_endpoint = global_state.image_endpoint.clone(); 
                        view! { cx,
                            img(src= format!("{}/{}", image_endpoint, x.image_name))
                        }
                    }
                )
                }
                div(style = "white-space: pre-line"){
                (if let Some(information) = &*props.recipe.information.get()
                {
                    let info = information.clone();
                    view! {cx, (info) }
                }
                else { view! {cx, }})}

                div {
                    h2 { "Ingredients" }

                    input(
                        placeholder = props.recipe.base_amount.get().to_string(),
                        type="number",
                        bind:value = amount
                    ) 
                    span { (props.recipe.unit_name) }

                    Indexed(
                        iterable = ingredients,
                        view = |cx, x| view! {cx,
                            p { (x.get()) }
                        }
                    )
                }
                div(style = "white-space: pre-line"){
                    (if let Some(preparation) = &*props.recipe.preparation.get()
                    {
                        let prep = preparation.clone();
                        view! { cx,
                            h2 { "Preparation" }
                            p { (prep)}
                        }
                    }
                    else { view! {cx, }})
                }
        }
    }
}

#[engine_only_fn]
async fn get_index_build_state(
    info: StateGeneratorInfo<()>,
) -> Result<RecipeProps, BlamedError<Error>> {
    let split = info.path.split("/");
    let resp = reqwest::get(format!(
        "{}longs/{}",
        &*BACKEND,
        split.last().unwrap_or_default()
    ))
    .await.map_err(Error::from)?
    .text()
    .await.map_err(Error::from)?;
    let resp = serde_json::from_str(&resp).map_err(Error::from)?;

    Ok(resp)
}

#[engine_only_fn]
async fn get_paths() -> Result<BuildPaths, Error> {
    let resp = reqwest::get(format!("{}/all_names", &*RECIPES_ENDPOINT))
        .await?
        .text()
        .await?;
    Ok(BuildPaths {
        paths: serde_json::from_str(&resp)?,
        extra: ().into(),
    })
}

#[derive(Serialize, Deserialize, Clone, ReactiveState)]
#[rx(alias = "RecipePropsRx")]
pub struct RecipeProps {
    #[rx(nested)]
    recipe: Recipe,
    ingredients: Vec<IngredientAndAmount>,
    images: Vec<Image>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Image {
    recipe_name: String,
    image_name: String,
    index: u32,
}