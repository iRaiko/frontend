use crate::{components::layout::Layout, DEFAULT_IMAGE_ENDPOINT, SHORTS_ENDPOINT, CATAGORIES_ENDPOINT};
use perseus::prelude::*;
use sycamore::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(ReactiveState, Serialize, Deserialize, Clone)]
#[rx(alias = "IndexPageStateRx")]
pub struct IndexPageState {
    pub catagories: Vec<Catagory>,
    recipes: Vec<Short>,
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
        div(class = "short-list") {
            ul
            {
                Indexed(
                    iterable=&props.recipes,
                    view=move |cx, x| view! { cx,
                    (if x.recipe.name.to_lowercase().contains(&*props.filters.get().to_lowercase())
                    {
                        let t = x.clone();
                        let name = t.recipe.name.clone();
                        let sig = create_signal(cx, t.ingredients);
                        let image_loc = x.image.clone();
                        view! { cx,
                        li {
                            (if let Some(loc) = &image_loc
                            {
                                let n = loc.clone();
                                view! { cx, td(style = "border: 1px solid #000000") { img(style = "width: 50px; height: 50px;", src=n)}}
                            }
                            else
                            {
                                view! { cx, td(style = "border: 1px solid #000000") { img(style = "width: 50px; height: 50px;", src=&props.default_img.get())}}
                            })
                            td(style = "border: 1px solid #000000; width: 100%; padding: 0;")
                            {
                                a(href= format!("recipe/{}", name), style = "display:block; text-decoration:none;") {
                                table(style = "border-collapse: collapse; width: 100%")
                                {
                                    tr(style = "border-bottom: 1px solid #000000; width: 100%;")
                                    {
                                        td(style = "text-align: center;") 
                                        { 
                                            p(style = "margin: 1px;") { (t.recipe.name) } 
                                            p(style = "margin: 1px;") { (format!("{} {}", t.recipe.base_amount, t.recipe.unit_name)) }
                                        }
                                    }
                                    tr
                                    {
                                        td {
                                            ul
                                            {
                                                Indexed(
                                                    iterable=sig,
                                                    view=|cx, x| view! {cx,
                                                    li { (format!("{} {} {}", x.amount, x.unit, x.name)) } }
                                                )
                                            }
                                        }
                                    }
                               }}
                            }
                        }
                    }}
                    else { view! { cx, }})}
                )
            }
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

#[derive(Serialize, Deserialize, Clone, PartialEq)]
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
