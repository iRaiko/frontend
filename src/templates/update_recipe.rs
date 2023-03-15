use perseus::{
    prelude::*,
    state::rx_collections::{RxVecNested, RxVecNestedRx},
};
use serde::{Deserialize, Serialize};
use sycamore::{prelude::*, rt::JsCast};
use web_sys::{ RequestMode, RequestInit,   Url, HtmlFormElement, HtmlDialogElement, HtmlInputElement, HtmlDivElement, HtmlImageElement};
use crate::{ BACKEND, UNITS_ENDPOINT, INGREDIENTS_ENDPOINT, CATAGORIES_ENDPOINT, RECIPES_ENDPOINT};
use crate::components::layout::Layout;
use web_sys::FormData as fd;
use std::borrow::Borrow;
use crate::errors::Error;
use crate::EndpointsRx;

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
                    let old_recipe = old_recipe.clone();
                    spawn_local_scoped(cx_t, async move 
                    {                                
                        let window = web_sys::window().unwrap();
                        let hydrate_node = file_input_ref.get::<HydrateNode>();
                        let input_element: HtmlInputElement = hydrate_node.unchecked_into();          
                        let files = input_element.files().unwrap(); // Safe to unwrap since the type = file
                        let mut opts = RequestInit::new();
                        opts.method("PATCH");
                        opts.mode(RequestMode::Cors);
                        let url = &global_state.image_endpoint.get();
        
                        let form = fd::new().unwrap();
                        for i in 0..files.length()
                        {
                            // form.append_with_blob_and_filename("file", &files.get(i).unwrap(), &files.get(i).unwrap().name());
                        }
                        opts.body(Some(form.as_ref()));
        
                        let request = web_sys::Request::new_with_str_and_init(&url, &opts).unwrap();
        
                        // let resp = window.fetch_with_request(&request);

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

                        // check the up to date list with the old list to see if there are any new entries and add them if there are
                        for i in &*t.ingredients
                        {
                            if !old_recipe.long.ingredients.contains(&i)
                            {
                                let resp = gloo_net::http::Request::post(&global_state.recipe_ingredient_endpoint.get()).mode(gloo_net::http::RequestMode::NoCors).json(&i).unwrap().send().await;
                            }
                            // let resp = gloo_net::http::Request::post(&add_ingredient_to_recipe_endpoint.get()).mode(gloo_net::http::RequestMode::NoCors).json(&i).unwrap().send().await.unwrap();
                        }
                        // check the old list if there are ingredients that are not in the up to date list and remove them if there are
                        for i in old_recipe.long.ingredients.iter()
                        {
                            if !t.ingredients.contains(&i)
                            {
                                // delete
                                let resp = gloo_net::http::Request::delete(&global_state.recipe_ingredient_endpoint.get()).mode(gloo_net::http::RequestMode::Cors).json(&i).unwrap().send().await;
                            }
                        }
                        for i in 0..files.length()
                        {
                            let recipe_image = Recipe_Image { recipe_name: recipe_name.clone(), image_name: files.get(i).unwrap().name() };
                            // let resp = gloo_net::http::Request::post(&add_image_to_recipe_endpoint.get()).mode(gloo_net::http::RequestMode::NoCors).json(&recipe_image).unwrap().send().await.unwrap();
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
                let list = document.create_element("ol").unwrap();

                while let Some(node) = &preview.first_child()
                {
                    &preview.remove_child(&node);
                }

                for i in 0..files.length()
                {
                    let list_item = document.create_element("li").unwrap();
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


#[component]
fn IngredientListCompotent<'a, G: Html>(cx: Scope<'a>, props: IngredientListProps<'a>) -> View<G> {
    view! { cx,
        div(class = "label-ingredient-div") {
            label(for = "information_text_area") { "Amount" }
            label(for = "information_text_area") { "Unit" }
            label(for = "information_text_area") { "Name" }
        }
        ul(style = "list-style-type: none; padding: 0")
        {
            Indexed(
                iterable = &props.ingredients,
                view = |cx, x|
                {
                    let amount_str = create_signal(cx, x.amount.get().to_string());
                    let amount_ref = create_ref(cx, x.amount);
                    let _ = create_effect(cx, ||
                        {
                            match amount_str.get().parse::<f32>()
                            {
                                Ok(float) => amount_ref.set(float),
                                Err(_) => amount_ref.set(1.)
                            };
                        });
                    let unit_ref = create_ref(cx, x.unit_name);
                    let name_ref = create_ref(cx, x.ingredient_name);
                    view! { cx,
                        li(style = "padding-bottom: 5px; display:flex;") {             
                            div(class = "ingredient-div") {
                                input(bind:value = amount_str, placeholder="Amount", required=true)
                            }
                            div(class = "ingredient-div") {
                                input(bind:value = unit_ref, placeholder = "Unit", list="unit_datalist", required = true)
                            }
                            div(class = "ingredient-div") {
                                input(bind:value = name_ref, placeholder = "Ingredient", list="ingredient_datalist", required = true)
                            }
                        }
                    }
                }
            )
        }
        button(type="button", on:click=
            {
                move |_|
                {
                    let ingredient = IngredientAndAmountPerseusRxIntermediate::new(props.recipe_name.clone(), "", 1.0, "");
                    props.ingredients.modify().push(ingredient);
                }
            }) { "Add Ingredient" }
        
        button(type="button", on:click=
            {
                move |_|
                {
                    props.ingredients.modify().pop();
                }
            }) { "Remove Ingredient" }
        }
    }
   

#[derive(Prop)]
struct IngredientListProps<'a> 
{   
    recipe_name: &'a RcSignal<String>,
    ingredients: &'a RxVecNestedRx<IngredientAndAmount>,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Catagory
{
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Ingredient
{
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Unit
{
    name: String,
}

#[derive(Serialize, Deserialize, Clone, ReactiveState, Debug)]
pub struct Long {
    #[rx(nested)]
    recipe: Recipe,
    #[rx(nested)]
    ingredients: RxVecNested<IngredientAndAmount>,
    images: Vec<Image>,
}

impl IngredientAndAmountPerseusRxIntermediate {
    fn new(
        recipe_name: RcSignal<String>,
        ingredient_name: &str,
        amount: f32,
        unit_name: &str,
    ) -> IngredientAndAmountPerseusRxIntermediate 
    {
        let ingredient_name = create_rc_signal(ingredient_name.to_owned());
        let amount = create_rc_signal(amount);
        let unit_name = create_rc_signal(unit_name.to_owned());
        IngredientAndAmountPerseusRxIntermediate {
            recipe_name,
            ingredient_name,
            amount,
            unit_name,
        }
    }
}

impl PartialEq for IngredientAndAmountPerseusRxIntermediate {
    fn eq(&self, other: &Self) -> bool {
        self.recipe_name.get() == other.recipe_name.get()
            && self.ingredient_name.get() == other.ingredient_name.get()
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Image {
    name: String,
    location: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, ReactiveState, Debug)]
pub struct IngredientAndAmount {
    recipe_name: String,
    ingredient_name: String,
    amount: f32,
    unit_name: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, ReactiveState, Debug)]
pub struct Recipe {
    name: String,
    catagory_name: String,
    information: Option<String>,
    base_amount: f32,
    unit_name: String,
    preparation: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct Recipe_Image
{
    recipe_name: String,
    image_name: String,
}