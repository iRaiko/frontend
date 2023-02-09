use perseus::prelude::*;
use serde::{Deserialize, Serialize};
use sycamore::prelude::*;
use crate::components::layout::Layout;
use crate::{RECIPES_ENDPOINT, BACKEND};

// Initialize our app with the `perseus_warp` package's default server (fully
// customizable)
pub fn get_template<G: Html>() -> Template<G> {
    Template::build("recipe")
        .build_paths_fn(get_paths)
        .build_state_fn(get_index_build_state)
        .revalidate_after("30s")
        .view_with_state(index_page)
        .incremental_generation()
        .build()
}

// EXCERPT_START
#[auto_scope]
fn index_page<G: Html>(cx: Scope, props: &IndexPropsRx) -> View<G> {
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

    view! { cx,
        Layout(title = "test")
        {
                h1 { (props.recipe.name)}
                Indexed(
                    iterable = &props.images,
                    view = |cx, x| view! { cx,
                        li { "haha yes" }
                    }
                )
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
                        min="0",
                        pattern=r"/\d+[[.,]?\d*]?/",
                        step="0.1",
                        bind:value = amount
                    ) { (props.recipe.unit_name) }

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

// This function will be run when you build your app, to generate default state
// ahead-of-time
#[engine_only_fn]
async fn get_index_build_state(
    info: StateGeneratorInfo<()>,
) -> Result<Long, BlamedError<reqwest::Error>> {
    let split = info.path.split("/");
    let resp = reqwest::get(format!(
        "{}longs/{}",
        &*BACKEND,
        split.last().unwrap()
    ))
    .await?
    .text()
    .await?;
    let resp = serde_json::from_str(&resp).unwrap();

    Ok(resp)
}
// EXCERPT_END

fn about_page<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        p { "This is an example webapp created with Perseus!" }
    }
}

#[engine_only_fn]
async fn get_paths() -> BuildPaths {
    let resp = reqwest::get(format!("{}/all_names", &*RECIPES_ENDPOINT))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    BuildPaths {
        paths: serde_json::from_str(&resp).unwrap(),
        extra: ().into(),
    }
}

#[derive(Serialize, Deserialize, Clone, ReactiveState)]
#[rx(alias = "IndexPropsRx")]
pub struct Long {
    #[rx(nested)]
    recipe: Recipe,
    ingredients: Vec<IngredientAndAmount>,
    images: Vec<Image>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Image {
    name: String,
    location: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct IngredientAndAmount {
    recipe_name: String,
    ingredient_name: String,
    amount: f32,
    unit_name: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, ReactiveState)]
pub struct Recipe {
    name: String,
    catagory_name: String,
    information: Option<String>,
    base_amount: f32,
    unit_name: String,
    preparation: Option<String>,
}
