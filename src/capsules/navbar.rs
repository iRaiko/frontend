use sycamore::prelude::*;
use perseus::prelude::*;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use crate::common::Catagory;

use crate::{CATAGORIES_ENDPOINT, LOGO_ENDPOINT};

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
    paths: Vec<String>,
    logo: String,
}


#[auto_scope]
fn navbar_capsule<G: Html>(cx: Scope, state: &NavbarStateRx, _props: ()) -> View<G>
{
    let node_ref = create_node_ref(cx);
    let mut is_closed = true;
    view ! {cx, 
        div(class = "navbar")
        {
            a(href = "") { img(src = &state.logo.get()) }

            div(class="navbar_links_closed", ref=node_ref){                    
                a(href = "") { "All" }
                a(href = "new") { "New" }
                a(href = "list") { "List" }
                Indexed(
                    iterable= &state.paths,
                    view=|cx,x|
                    {
                        let y = x.clone();
                        view! { cx,
                            a(href = y) { (x) }
                        }
                    }
                )
            }            
            button(on:click = move |_|
            {
                let node1 = node_ref.try_get::<HydrateNode>();
                if let Some(node) = node1{
                if is_closed
                {
                    node.set_class_name("navbar_links_open");
                    is_closed = false;
                }
                else
                {
                    node.set_class_name("navbar_links_closed");
                    is_closed = true;
                }}
            }) { "Menu" }
        }   
    }
}

#[engine_only_fn]
async fn get_build_state(_info: StateGeneratorInfo<()>) -> Result<NavbarState, BlamedError<reqwest::Error>>
{
    let resp = reqwest::get(&*CATAGORIES_ENDPOINT)
        .await?
        .text()
        .await?;
    let catagories: Vec<Catagory> = serde_json::from_str(&resp).unwrap();
    let paths: Vec<String> = catagories.into_iter().map(|x| x.name).collect();
    Ok(NavbarState { paths, logo: (LOGO_ENDPOINT).to_string() })
}
