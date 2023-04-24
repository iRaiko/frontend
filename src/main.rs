mod components;
mod error_views;
mod capsules;
mod errors;
mod templates;
mod common;
use std::env::VarError;

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
    pub static ref RECIPE_INGREDIENTS_ENDPOINT: String = [&RECIPES_ENDPOINT, "/ingredients"].concat();
    pub static ref RECIPE_IMAGES_ENDPOINT: String = [&RECIPES_ENDPOINT, "/image"].concat();
    pub static ref CATAGORIES_ENDPOINT: String = [&BACKEND, "catagories"].concat();
    pub static ref IMAGES_ENDPOINT: String = [&BACKEND, "images"].concat();
    pub static ref DEFAULT_IMAGE_ENDPOINT: String = [&IMAGES_ENDPOINT, "/", &DEFAULT_IMAGE].concat();
    pub static ref LOGO_ENDPOINT: String = [&IMAGES_ENDPOINT, "/", &LOGO].concat();
}

pub fn get_global_state_creator() -> GlobalStateCreator
{
    GlobalStateCreator::new()
        .build_state_fn(get_build_state)
}

#[derive(Serialize, Deserialize, ReactiveState, Clone)]
#[rx(alias = "EndpointsRx")]
pub struct Endpoints
{
    pub default_image: String,
    pub image_endpoint: String,
    pub units_endpoint: String,
    pub recipe_endpoint: String,
    pub catagories_endpoint: String,
    pub ingredients_endpoint: String,
    pub recipe_ingredient_endpoint: String,
    pub recipe_image_endpoint: String,
    #[rx(nested)]
    pub app_state: AppState,
}

#[derive(Serialize, Deserialize, ReactiveState, Clone)]
#[rx(alias = "AppStateRx")]
pub struct AppState
{
    pub state: bool,
    pub session_key: Option<String>,
}

#[cfg(client)]
impl AppStateRx
{
    pub async fn verify_key(&self)
    {
        if let Some(key) = &*self.session_key.get()
        {
            let response = gloo_net::http::Request::post("http://192.168.68.105:8000/session/").body(key).mode(gloo_net::http::RequestMode::Cors).send().await;
            if let Ok(response_body) = response
            {
                if response_body.ok()
                {
                    self.state.set(true)
                }
            }
            else {
                self.state.set(false)
            }
        }
        else {
            self.state.set(false)
        }
    }
}

#[engine_only_fn]
async fn get_build_state(_locale: String) -> Result<Endpoints, VarError>
{
    Ok(Endpoints
    {
        default_image: DEFAULT_IMAGE_ENDPOINT.to_string(),
        image_endpoint: IMAGES_ENDPOINT.to_string(),
        catagories_endpoint: CATAGORIES_ENDPOINT.to_string(),
        recipe_endpoint: RECIPES_ENDPOINT.to_string(),
        ingredients_endpoint: INGREDIENTS_ENDPOINT.to_string(),
        units_endpoint: UNITS_ENDPOINT.to_string(),
        recipe_ingredient_endpoint: RECIPE_INGREDIENTS_ENDPOINT.to_string(),
        recipe_image_endpoint: RECIPE_IMAGES_ENDPOINT.to_string(),
        app_state: AppState { state: false, session_key: None},
    })
}

#[perseus::main(perseus_axum::dflt_server)]
pub fn main<G: Html>() -> PerseusApp<G> {
    dotenv().ok();
    PerseusApp::new()
        .error_views(crate::error_views::get_error_views())
        .template(crate::templates::shorts::get_template())
        .template(crate::templates::recipe::get_template())
        .template(crate::templates::new_recipe::get_template())
        .template(crate::templates::update_recipe::get_template())
        .capsule_ref(&*crate::capsules::navbar::NAVBAR)
        .global_state_creator(get_global_state_creator())
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