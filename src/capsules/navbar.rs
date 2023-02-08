use sycamore::prelude::*;
use perseus::prelude::*;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

const KEUR_IMAGE: &str = "http://127.0.0.1:8000/images/keur.jpg";

lazy_static!
{
    pub static ref NAVBAR: Capsule<PerseusNodeType, ()> = get_capsule();
}

pub fn get_capsule<G: Html>() -> Capsule<G, ()>
{
    Capsule::build(Template::build("navbar").build_state_fn(get_build_state))
        .empty_fallback()
        .view_with_state(navbar_capsule)
        .build()
}

#[derive(Serialize, Deserialize, Clone, ReactiveState)]
#[rx(alias = "NavbarStateRx")]
struct NavbarState
{
    paths: Vec<String>
}


#[auto_scope]
fn navbar_capsule<G: Html>(cx: Scope, state: &NavbarStateRx, props: ()) -> View<G>
{
    view ! {cx, 
        nav(class = "navbar")
        {
            a(href = "") { img(src = KEUR_IMAGE) }
            ul{                    
                li { a(href = "") { "All" }}
                li { a(href = "new") { "New" }}
                li { a(href = "list") { "List" }}
                Indexed(
                    iterable= &state.paths,
                    view=|cx,x|
                    {
                        let y = x.clone();
                        view! { cx,
                            li { a(href = y) { (x) } } 
                        }
                    }
                )
            }
        }   
    }
}

#[engine_only_fn]
async fn get_build_state(_info: StateGeneratorInfo<()>) -> Result<NavbarState, BlamedError<reqwest::Error>>
{
    let resp = reqwest::get("http://127.0.0.1:8000/catagories")
        .await?
        .text()
        .await?;
    let catagories: Vec<Catagory> = serde_json::from_str(&resp).unwrap();
    let mut paths: Vec<String> = catagories.into_iter().map(|x| x.name).collect();
    Ok(NavbarState { paths })
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Hash, Eq)]
pub struct Catagory {
    name: String,
}