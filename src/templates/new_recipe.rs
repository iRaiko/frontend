use perseus::{
    prelude::*,
    state::rx_collections::{RxVecNested, RxVecNestedRx},
};
use serde::{Deserialize, Serialize};
use sycamore::{prelude::*, rt::JsCast};
use web_sys::{RequestMode, RequestInit, FileReader, Node, Url, HtmlFormElement, HtmlDialogElement, HtmlInputElement, HtmlDivElement, HtmlImageElement};
use crate::{capsules::navbar::NAVBAR, UNITS_ENDPOINT, INGREDIENTS_ENDPOINT, CATAGORIES_ENDPOINT, RECIPES_ENDPOINT, RECIPE_INGREDIENTS_ENDPOINT, IMAGES_ENDPOINT, ADD_IMAGES_ENDPOINT, components::ingredient_list::IngredientListCompotent, common::{IngredientAndAmount, Ingredient, Catagory, Unit, Recipe}};
use crate::components::layout::Layout;
use web_sys::FormData as fd;
use std::borrow::Borrow;
use crate::errors::Error;

pub fn get_template<G: Html>() -> Template<G> {
    Template::build("new")
        .request_state_fn(get_form_build_state)
        .view_with_state(form_page)
        .build()
}

#[auto_scope]
fn form_page<G: Html>(cx: Scope, props: &mut FormDataRx) -> View<G> 
{
    let FormDataRx { 
        long: props, 
        ingredients, 
        units , 
        catagories,
        catagories_endpoint,
        units_endpoint,
        ingredients_endpoint,
        recipes_endpoint,
        add_ingredient_to_recipe_endpoint,
        images_endpoint,
        add_image_to_recipe_endpoint,
    } = props;
    let name_ref = create_ref(cx, &props.recipe.name);
    let catagory_ref = create_ref(cx, &props.recipe.catagory_name);
    let amount_str = create_signal(cx, props.recipe.base_amount.get().to_string());
    let amount_ref = create_ref(cx, &props.recipe.base_amount);
    let amount_memo = create_effect(cx, || {
        match amount_str.get().parse::<f32>() {
            Ok(float) => amount_ref.set(float),
            Err(e) => amount_ref.set(1.),
        };
    });
    let information_str = create_signal(cx, "".to_string());
    let information_memo = create_effect(cx, || {
        props
            .recipe
            .information
            .set(if !information_str.get().is_empty() {
                Some(information_str.get().to_string())
            } else {
                None
            });
    });
    let preparation_str = create_signal(cx, "".to_string());
    let preparation_memo = create_effect(cx, || {
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

    let test = create_signal(cx, "".to_string());

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
    let file_input_ref = create_node_ref(cx);

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
                    spawn_local_scoped(cx_t, async move 
                    {                                
                        let window = web_sys::window().unwrap();
                        let hydrate_node = file_input_ref.get::<HydrateNode>();
                        let input_element: HtmlInputElement = hydrate_node.unchecked_into();          
                        let files = input_element.files().unwrap(); // Safe to unwrap since the type = file
                        let mut opts = RequestInit::new();
                        opts.method("POST");
                        opts.mode(RequestMode::Cors);
                        let url = "http://192.168.68.100:8000/images";
        
                        let form = fd::new().unwrap();
                        for i in 0..files.length()
                        {
                            form.append_with_blob_and_filename("file", &files.get(i).unwrap(), &files.get(i).unwrap().name());
                        }
                        opts.body(Some(form.as_ref()));
        
                        let request = web_sys::Request::new_with_str_and_init(&url, &opts).unwrap();
        
                        let resp = window.fetch_with_request(&request);

                        if !catagories.get().iter().any(|x| x.name == *props.recipe.catagory_name.get())
                        {
                            let catagory = Catagory { name: (*props.recipe.catagory_name.get()).clone()};
                            let resp = gloo_net::http::Request::post(&catagories_endpoint.get()).mode(gloo_net::http::RequestMode::NoCors).json(&catagory).unwrap().send().await.unwrap();
                        }        
                        for i in &*new_ingredients_signal.get()
                        {
                            let ingredient = Ingredient { name: i.clone()};
                            let resp = gloo_net::http::Request::post(&ingredients_endpoint.get()).mode(gloo_net::http::RequestMode::NoCors).json(&ingredient).unwrap().send().await.unwrap();
                        }
                        for i in &*new_units_signal.get()
                        {
                            let unit = Unit { name: i.clone()};
                            let resp = gloo_net::http::Request::post(&units_endpoint.get()).mode(gloo_net::http::RequestMode::NoCors).json(&unit).unwrap().send().await.unwrap();
                        }
                        use perseus::state::MakeUnrx;
                        let t = props.clone().make_unrx();
                        let recipe_name = props.clone().make_unrx().recipe.name;
                        let resp = gloo_net::http::Request::post(&recipes_endpoint.get()).mode(gloo_net::http::RequestMode::NoCors).json(&t.recipe).unwrap().send().await.unwrap();
                        for i in &*t.ingredients
                        {
                            let resp = gloo_net::http::Request::post(&add_ingredient_to_recipe_endpoint.get()).mode(gloo_net::http::RequestMode::NoCors).json(&i).unwrap().send().await.unwrap();
                        }
                        for i in 0..files.length()
                        {
                            let recipe_image = Recipe_Image { recipe_name: recipe_name.clone(), image_name: files.get(i).unwrap().name() };
                            let resp = gloo_net::http::Request::post(&add_image_to_recipe_endpoint.get()).mode(gloo_net::http::RequestMode::NoCors).json(&recipe_image).unwrap().send().await.unwrap();
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
            (dialog)
            (unit_datalist)
            (ingredient_datalist)
            (catagory_datalist)
            div(class = "form") {
            form(id = "recipe_form") {
            div(class = "form-row") {
                div(class = "form-block") {
                    label(for = "recipe_name") { "Recipe Name" }
                    input(id = "recipe_name", bind:value = name_ref, placeholder = "recipe name", required = true)
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
                div(class = "form-block") {
                    IngredientListCompotent(recipe_name = &props.recipe.name, ingredients=&props.ingredients)
                }
            }
            div(class = "form-row") {
            input(ref=file_input_ref, type="file", accept="image/*", multiple=true, id="image_uploads", on:change = move |_|
            {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let input: HtmlInputElement = document.get_element_by_id("image_uploads").unwrap().dyn_into().unwrap();
                let preview: HtmlDivElement = document.get_element_by_id("preview").unwrap().dyn_into().unwrap();
                let files = input.files().unwrap(); // Can unwrap here because input is of type file
                let mut list = document.create_element("ol").unwrap();

                while let Some(node) = &preview.first_child()
                {
                    &preview.remove_child(&node);
                }

                for i in 0..files.length()
                {
                    let mut list_item = document.create_element("li").unwrap();
                    let para = document.create_element("p").unwrap();
                    para.set_text_content(Some(&files.get(i).unwrap().name()));

                    let image: HtmlImageElement = document.create_element("img").unwrap().dyn_into().unwrap();
                    image.set_src(&Url::create_object_url_with_blob(&files.get(i).unwrap()).unwrap());

                    image.set_class_name("preview_image");

                    list_item.append_child(&para);
                    list_item.append_child(&image);
                    list.append_child(&list_item);
                }


                preview.append_child(&list);
            })
                div(id = "preview"){}
            }
            button(type = "button", on:click = move |_| 
            {
                
                let form: HtmlFormElement = web_sys::window().unwrap().document().unwrap().get_element_by_id("recipe_form").unwrap().dyn_into().unwrap();
                if form.report_validity()
                {
                    let test = dialog_ref.get::<HydrateNode>();
                    let dialog_element: HtmlDialogElement = test.unchecked_into();
                    dialog_element.show_modal();
                }
            }) { "Submit" }
        }}
    }
}
}

#[engine_only_fn]
async fn get_form_build_state(_: StateGeneratorInfo<()>, _req: Request) -> Result<FormData, BlamedError<crate::errors::Error>>
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

    let long = Long {
        recipe: Recipe {
            name: "".to_string(),
            catagory_name: "".to_string(),
            information: None,
            base_amount: 1.,
            unit_name: "".to_string(),
            preparation: None,
        },
        ingredients: vec![].into(),
        images: vec![],
    };

    Ok(FormData
    {
        long,
        ingredients,
        units,
        catagories,
        catagories_endpoint: CATAGORIES_ENDPOINT.to_string(),
        units_endpoint: UNITS_ENDPOINT.to_string(),
        recipes_endpoint: RECIPES_ENDPOINT.to_string(),
        ingredients_endpoint: INGREDIENTS_ENDPOINT.to_string(),
        add_ingredient_to_recipe_endpoint: RECIPE_INGREDIENTS_ENDPOINT.to_string(),
        images_endpoint: IMAGES_ENDPOINT.to_string(),
        add_image_to_recipe_endpoint: ADD_IMAGES_ENDPOINT.to_string(),
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
    
    catagories_endpoint: String,
    units_endpoint: String,
    recipes_endpoint: String,
    ingredients_endpoint: String,
    images_endpoint: String,
    add_ingredient_to_recipe_endpoint: String,
    add_image_to_recipe_endpoint: String,
}

#[derive(Serialize, Deserialize, Clone, ReactiveState, Debug)]
pub struct Long {
    #[rx(nested)]
    recipe: Recipe,
    #[rx(nested)]
    ingredients: RxVecNested<IngredientAndAmount>,
    images: Vec<Image>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Image {
    name: String,
    location: String,
}

#[derive(Serialize, Deserialize)]
pub struct Recipe_Image
{
    recipe_name: String,
    image_name: String,
}