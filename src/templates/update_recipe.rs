use perseus::{
    prelude::*,
    state::rx_collections::{RxVecNested, RxVecNestedRx},
};
use serde::{Deserialize, Serialize};
use sycamore::{prelude::*, rt::JsCast};
use web_sys::{ RequestMode, RequestInit,   Url, HtmlFormElement, HtmlDialogElement, HtmlInputElement, HtmlDivElement, HtmlImageElement, File, Blob};
use crate::{ BACKEND, UNITS_ENDPOINT, INGREDIENTS_ENDPOINT, CATAGORIES_ENDPOINT, RECIPES_ENDPOINT, components::{ingredient_list::IngredientListCompotent, login::Login, file_selector::FileSelector}, common::{Recipe, Ingredient, Unit, Catagory, Image}};
use crate::components::layout::Layout;
use web_sys::FormData as fd;
use std::borrow::Borrow;
use crate::errors::Error;
use crate::EndpointsRx;

use crate::common::{IngredientAndAmount, IngredientAndAmountPerseusRxIntermediate};

pub fn get_template<G: Html>() -> Template<G> {
    Template::build("update")
        .request_state_fn(get_form_build_state)
        .build_paths_fn(get_paths)
        .view_with_state(form_page)
        .build()
}

#[auto_scope]
fn form_page<G: Html>(cx: Scope, props: &mut FormDataRx) -> View<G> 
{
    let global_state = Reactor::<G>::from_cx(cx).get_global_state::<EndpointsRx>(cx);

    use perseus::state::MakeUnrx;
    let old_recipe = props.clone().make_unrx();

    let FormDataRx { 
        long: props, 
        ingredients, 
        units , 
        catagories,
    } = props;


    let name_ref = create_ref(cx, &props.recipe.name);
    let catagory_ref = create_ref(cx, &props.recipe.catagory_name);
    let amount_str = create_signal(cx, props.recipe.base_amount.get().to_string());
    let amount_ref = create_ref(cx, &props.recipe.base_amount);
    create_effect(cx, || {
        match amount_str.get().parse::<f32>() {
            Ok(float) => amount_ref.set(float),
            Err(_) => amount_ref.set(1.),
        };
    });
    let information_str = create_signal(cx,         
        {
        if let Some(info) = props.recipe.information.get().borrow()
        {
            info.to_owned()
        }
        else
        {
            "".to_string()
        }
    });
    create_effect(cx, || {
        props
            .recipe
            .information
            .set(if !information_str.get().is_empty() {
                Some(information_str.get().to_string())
            } else {
                None
            });
    });
    let preparation_str = create_signal(cx, 
        {
            if let Some(info) = props.recipe.preparation.get().borrow()
            {
                info.to_owned()
            }
            else
            {
                "".to_string()
            }
        }
    );
    create_effect(cx, || {
        props
            .recipe
            .preparation
            .set(if !preparation_str.get().is_empty() {
                Some(preparation_str.get().to_string())
            } else {
                None
            });
    });
    let unit_ref = create_ref(cx, &props.recipe.unit_name);

    let cx_t = cx.clone();

    let unit_datalist = view ! { cx, 
        datalist(id="unit_datalist")
        { Indexed(
            iterable=units,
            view = |cx, x| view! {cx, 
                option(value=x.name)
            }
        )} 
    };

    let catagory_datalist = view ! { cx, 
        datalist(id="catagory_datalist")
        { Indexed(
            iterable=catagories,
            view = |cx, x| view! {cx, 
                option(value=x.name)
            }
        )} 
    };

    let ingredient_datalist = view ! { cx, 
        datalist(id="ingredient_datalist")
        { Indexed(
            iterable=ingredients,
            view = |cx, x| view! {cx, 
                option(value=x.name)
            }
        )} 
    };
  
    let new_catagories = view! { cx,
        (if !catagories.get().iter().any(|x| x.name == *props.recipe.catagory_name.get())        
        {
            view! {cx, 
                h1 { "A new catagory will be created:"}
                (props.recipe.catagory_name.get())
            }
        }
        else
        {
            view! {cx, }
        })
    };

    let new_units_signal = create_memo(cx, ||
        {
            let mut new = Vec::new();
            if !units.get().iter().any(|x| x.name == *props.recipe.unit_name.get())
            {
                let name: String = (*props.recipe.unit_name.get()).clone();
                new.push(name);
            }
            for i in props.ingredients.get().iter()
            {
                let name = (*i.unit_name.get()).clone();
                if !units.get().iter().any(|x| x.name == name)
                {
                    new.push(name);
                }
            }
            new.dedup();
            new
        }
    );

    let new_units = view! {cx, 
        
        (if !new_units_signal.get().is_empty()
        {
            view! {cx, h1 { "New units will be inserted:"}
            Indexed(
                iterable = new_units_signal,
                view = |cx, x|
                view! { cx, 
                    (x)
                }
            )}
        }
        else
        {
            view! {cx, }
        })
    };
    let new_ingredients_signal = create_memo(cx, ||
        {
            let mut new = Vec::new();
            for i in props.ingredients.get().iter()
            {
                let name = (*i.ingredient_name.get()).clone();
                if !ingredients.get().iter().any(|x| x.name == name)
                {
                    new.push(name);
                }
            }
            new.dedup();
            new
        }
    );

    let new_ingredients = view! {cx, 
        
        (if !new_ingredients_signal.get().is_empty()
        {
            view! {cx, h1 { "New ingredients will be inserted:"}
            Indexed(
                iterable = new_ingredients_signal,
                view = |cx, x|
                view! { cx, 
                    p { (x) }
                }
            )}
        }
        else
        {
            view! {cx, }
        })
    };

    let dialog_ref = create_node_ref(cx);

    let dialog = view! { cx,
        dialog(ref=dialog_ref, id = "confirm_dialog") {
            div
            {
                (new_catagories)
                (new_units)
                (new_ingredients)
            }
            button(on:click = move |_|
                {
                    let old_recipe = old_recipe.clone();
                    spawn_local_scoped(cx_t, async move 
                    {                                
                        if !catagories.get().iter().any(|x| x.name == *props.recipe.catagory_name.get())
                        {
                            let catagory = Catagory { name: (*props.recipe.catagory_name.get()).clone()};
                            let resp = gloo_net::http::Request::post(&global_state.catagories_endpoint.get()).mode(gloo_net::http::RequestMode::NoCors).json(&catagory).unwrap().send().await.unwrap();
                        }        
                        for i in &*new_ingredients_signal.get()
                        {
                            let ingredient = Ingredient { name: i.clone()};
                            let resp = gloo_net::http::Request::post(&global_state.ingredients_endpoint.get()).mode(gloo_net::http::RequestMode::NoCors).json(&ingredient).unwrap().send().await.unwrap();
                        }
                        for i in &*new_units_signal.get()
                        {
                            let unit = Unit { name: i.clone()};
                            let resp = gloo_net::http::Request::post(&global_state.units_endpoint.get()).mode(gloo_net::http::RequestMode::NoCors).json(&unit).unwrap().send().await.unwrap();
                        }
                        use perseus::state::MakeUnrx;
                        let t = props.clone().make_unrx();
                        let recipe_name = props.clone().make_unrx().recipe.name;
                        let resp = gloo_net::http::Request::patch(&global_state.recipe_endpoint.get()).mode(gloo_net::http::RequestMode::Cors)
                            .json(&t.recipe).unwrap().send().await;

                        for i in &*t.images
                        {
                            if let Some(image_url) = &i.data
                            {
                                let window = web_sys::window().unwrap();
                                let response: web_sys::Response = sycamore::futures::JsFuture::from(window.fetch_with_str(&image_url)).await.unwrap().unchecked_into();
                                let blob: web_sys::Blob = sycamore::futures::JsFuture::from(response.blob().unwrap()).await.unwrap().unchecked_into();
                                let mut opts = RequestInit::new();
                                opts.method("POST");
                                opts.mode(RequestMode::Cors);
                                let url = &global_state.image_endpoint.get();
                
                                let form = fd::new().unwrap();
                                form.append_with_blob_and_filename("file", &blob, &i.image_name);

                                opts.body(Some(form.as_ref()));
                                let request = web_sys::Request::new_with_str_and_init(&url, &opts).unwrap();
                
                                let resp = window.fetch_with_request(&request);
                            }
                        }
                        for i in old_recipe.long.ingredients.iter()
                        {
                            let resp = gloo_net::http::Request::delete(&global_state.recipe_ingredient_endpoint.get()).mode(gloo_net::http::RequestMode::Cors).json(&i).unwrap().send().await;
                        }

                        for i in &*t.ingredients
                        {
                            let resp = gloo_net::http::Request::post(&global_state.recipe_ingredient_endpoint.get()).mode(gloo_net::http::RequestMode::NoCors).json(&i).unwrap().send().await;
                        }
                        
                        for i in old_recipe.long.images.iter()
                        {
                            let resp = gloo_net::http::Request::delete(&global_state.recipe_image_endpoint.get()).mode(gloo_net::http::RequestMode::Cors).json(&i).unwrap().send().await.unwrap();
                        }

                        for i in &*t.images
                        {
                            let resp = gloo_net::http::Request::post(&global_state.recipe_image_endpoint.get()).mode(gloo_net::http::RequestMode::Cors).json(&i).unwrap().send().await.unwrap();
                        }
                    });
                }
            ) { "Submit" }
            button(on:click = |_|
                {                       
                    let hydrate_node = dialog_ref.get::<HydrateNode>();
                    let dialog_element: HtmlDialogElement = hydrate_node.unchecked_into();              
                    dialog_element.close();
                }
            ) { "Cancel" }
        }
    };

    view! {cx,
        Layout(title = "stuff") 
        {
            Login(state = &global_state.app_state)
            {
            (dialog)
            (unit_datalist)
            (ingredient_datalist)
            (catagory_datalist)
            div(class = "form") {
            form(id = "recipe_form") {
            div(class = "form-row") {
                div(class = "form-block") {
                    label(for = "recipe_name") { "Recipe Name" }
                    input(id = "recipe_name", bind:value = name_ref, placeholder = "recipe name", required = true, readonly = true)
                }
                div(class = "form-block") {
                    label(for = "catagory_name") { "Catagory Name" }
                    input(id = "catagory_name", bind:value = catagory_ref, placeholder = "catagory name", required = true, list = "catagory_datalist")
                }
            }
            div(class = "form-row") {
                div(class = "form-block") {
                    label(for = "information_text_area") { "Information" }
                    textarea(id = "information_text_area", class = "textareaElement", bind:value = information_str, placeholder = "information", contenteditable=true)
                }
            }
            div(class = "form-row") {
                div(class = "form-block") {
                    label(for = "preparation_text_area") { "Preparation" }
                    textarea(id = "preperation_text_area", class = "textareaElement", bind:value = preparation_str, placeholder = "preparation", contenteditable=true)
                }
            }
            div(class = "form-row") {
                div(class = "form-block") {
                    label(for = "recipe_amount_input") { "Base Amount" }
                    input(id = "recipe_amount_input", bind:value = amount_str, required=true)
                }
                div(class = "form-block") {
                    label(for = "recipe_unit_input") { "Unit Name" }
                    input(id = "recipe_unit_input", bind:value = unit_ref, placeholder = "unit name", list="unit_datalist", required=true)
                }
            }
            div(class = "form-row") {
                div(class = "form-block") 
                {
                    IngredientListCompotent(recipe_name = &props.recipe.name, ingredients=&props.ingredients)
                }
            }
            div(class = "form-row") {
                div(class = "form-block")
                {
                    FileSelector(recipe_name = &props.recipe.name, images=&props.images)
                }
            }
            button(type = "button", on:click = move |_| 
            {
                
                let form: HtmlFormElement = web_sys::window().unwrap().document().unwrap().get_element_by_id("recipe_form").unwrap().dyn_into().unwrap();
                if form.report_validity()
                {
                    let test = dialog_ref.get::<HydrateNode>();
                    let dialog_element: HtmlDialogElement = test.unchecked_into();
                    let _ = dialog_element.show_modal();
                }
            }) { "Submit" }
        }}
    }
    }
}
}

#[engine_only_fn]
async fn get_form_build_state(info: StateGeneratorInfo<()>, _req: Request) -> Result<FormData, BlamedError<crate::errors::Error>>
{
    let resp = reqwest::get(&*CATAGORIES_ENDPOINT)
        .await.map_err(Error::from)?
        .text()
        .await.map_err(Error::from)?;
    let catagories: Vec<Catagory> = serde_json::from_str(&resp).map_err(Error::from)?;

    let resp = reqwest::get(&*INGREDIENTS_ENDPOINT)
        .await.map_err(Error::from)?
        .text()
        .await.map_err(Error::from)?;
    let ingredients: Vec<Ingredient> = serde_json::from_str(&resp).map_err(Error::from)?;

    let resp = reqwest::get(&*UNITS_ENDPOINT)
        .await.map_err(Error::from)?
        .text()
        .await.map_err(Error::from)?;
    let units: Vec<Unit> = serde_json::from_str(&resp).map_err(Error::from)?;

    let split = info.path.split("/");
    let resp = reqwest::get(format!(
        "{}longs/{}",
        &*BACKEND,
        split.last().unwrap_or_default()
    ))
    .await.map_err(Error::from)?
    .text()
    .await.map_err(Error::from)?;
    let long = serde_json::from_str(&resp).map_err(Error::from)?;

    Ok(FormData
    {
        long,
        ingredients,
        units,
        catagories,
    })
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

#[derive(Serialize, Deserialize, Clone, ReactiveState, Debug)]
#[rx(alias = "FormDataRx")]
pub struct FormData
{
    #[rx(nested)]
    long: Long,
    ingredients: Vec<Ingredient>,
    units: Vec<Unit>,
    catagories: Vec<Catagory>,
}

#[derive(Serialize, Deserialize, Clone, ReactiveState, Debug)]
pub struct Long {
    #[rx(nested)]
    recipe: Recipe,
    #[rx(nested)]
    ingredients: RxVecNested<IngredientAndAmount>,
    #[rx(nested)]
    images: RxVecNested<ImageFile>,
}

#[derive(Serialize, Deserialize)]
pub struct RecipeImage
{
    recipe_name: String,
    image_name: String,
    index: u32,
}

#[derive(Serialize, Deserialize, ReactiveState, Clone, Debug)]
pub struct ImageFile
{
    pub recipe_name: String,
    pub image_name: String,    
    pub index: u32,
    #[serde(skip)]
    pub data: Option<String>,
}
