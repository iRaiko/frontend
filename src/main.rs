mod components;
mod error_views;
mod capsules;
mod templates;
use clap::Parser;
use serde::{Serialize, Deserialize};
use perseus::{prelude::*, state::GlobalStateCreator};
use lazy_static::lazy_static;

#[derive(Parser)]
pub struct Cli
{
    backend_ip: String,
    backend_port: String,
}

lazy_static!
{
    pub static ref BACKEND_IP: Cli = Cli::parse();
}


#[perseus::main(perseus_warp::dflt_server)]
pub fn main<G: Html>() -> PerseusApp<G> {
    PerseusApp::new()
        .error_views(crate::error_views::get_error_views())
        .template(crate::templates::shorts::get_template())
        .template(crate::templates::recipe::get_template())
        .template(crate::templates::new_recipe::get_template())
        .capsule_ref(&*crate::capsules::navbar::NAVBAR)
        .index_view(|cx|
        {
            sycamore::view! { cx, 
            head {
                title { "Keurslagerij Recepten" }
                meta(name="viewport",  content="width=device-width, initial-scale=1.0")
                link(rel = "stylesheet", href =".perseus/static/style.css")
            }
            body {
                PerseusRoot()
            }
        }
        }
    )
}