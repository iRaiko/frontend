mod components;
mod error_views;
mod capsules;
mod templates;
use dotenvy::dotenv;
use serde::{Serialize, Deserialize};
use perseus::{prelude::*, state::GlobalStateCreator};
use lazy_static::lazy_static;

lazy_static!
{
    pub static ref BACKEND_IP: String = std::env::var("BACKEND_IP").expect("Enviroment Variable 'BACKEND_IP' not found");
    pub static ref BACKEND_PORT: String = std::env::var("BACKEND_PORT").expect("Enviroment Variable 'BACKEND_PORT' not found");
    pub static ref DEFAULT_IMAGE: String = std::env::var("DEFAULT_IMAGE").expect("Enviroment Variable 'DEFAULT_IMAGE' not found");
    pub static ref LOGO: String = std::env::var("LOGO").expect("Enviroment Variable 'LOGO' not found");

    pub static ref BACKEND: String = ["http://", &BACKEND_IP, ":", &BACKEND_PORT, "/"].concat();
    pub static ref UNITS_ENDPOINT: String = [&BACKEND, "units"].concat();
    pub static ref SHORTS_ENDPOINT: String = [&BACKEND, "shorts"].concat();
    pub static ref INGREDIENTS_ENDPOINT: String = [&BACKEND, "ingredients"].concat();
    pub static ref RECIPES_ENDPOINT: String = [&BACKEND, "recipes"].concat();
    pub static ref ADD_INGREDIENTS_ENDPOINT: String = [&RECIPES_ENDPOINT, "/add_ingredient"].concat();
    pub static ref CATAGORIES_ENDPOINT: String = [&BACKEND, "catagories"].concat();
    pub static ref IMAGES_ENDPOINT: String = [&BACKEND, "images"].concat();
    pub static ref DEFAULT_IMAGE_ENDPOINT: String = [&IMAGES_ENDPOINT, "/", &DEFAULT_IMAGE].concat();
    pub static ref LOGO_ENDPOINT: String = [&IMAGES_ENDPOINT, "/", &LOGO].concat();
}


#[perseus::main(perseus_warp::dflt_server)]
pub fn main<G: Html>() -> PerseusApp<G> {
    dotenv().ok();
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