use perseus::{
    prelude::*,
    state::rx_collections::{RxVecNested, RxVecNestedRx},
};
use serde::{Deserialize, Serialize};
use sycamore::{prelude::*, rt::JsCast};
use web_sys::{HtmlFormElement, HtmlDialogElement};
use crate::capsules::navbar::NAVBAR;
use crate::components::layout::Layout;
use std::borrow::Borrow;

pub fn get_template<G: Html>() -> Template<G> {
    Template::build("new")
        .request_state_fn(get_form_build_state)
        .view_with_state(form_page)
        .build()
}

#[auto_scope]
fn form_page<G: Html>(cx: Scope, props: &mut FormDataRx) -> View<G> 
{
    let FormDataRx { long: props, ingredients, units , catagories} = props;
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
    let temp = create_memo(cx, || information_str.get());
    let temp_ref = create_ref(cx, temp);
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
                        if !catagories.get().iter().any(|x| x.name == *props.recipe.catagory_name.get())
                        {
                            let catagory = Catagory { name: (*props.recipe.catagory_name.get()).clone()};
                            let resp = gloo_net::http::Request::post("http://127.0.0.1:8000/catagories").mode(gloo_net::http::RequestMode::NoCors).json(&catagory).unwrap().send().await.unwrap();
                        }        
                        for i in &*new_ingredients_signal.get()
                        {
                            let ingredient = Ingredient { name: i.clone()};
                            let resp = gloo_net::http::Request::post("http://127.0.0.1:8000/ingredients").mode(gloo_net::http::RequestMode::NoCors).json(&ingredient).unwrap().send().await.unwrap();
                        }
                        for i in &*new_units_signal.get()
                        {
                            let unit = Unit { name: i.clone()};
                            let resp = gloo_net::http::Request::post("http://127.0.0.1:8000/units").mode(gloo_net::http::RequestMode::NoCors).json(&unit).unwrap().send().await.unwrap();
                        }
                        use perseus::state::MakeUnrx;
                        let t = props.clone().make_unrx();
                        let resp = gloo_net::http::Request::post("http://127.0.0.1:8000/recipes").mode(gloo_net::http::RequestMode::NoCors).json(&t.recipe).unwrap().send().await.unwrap();
                        for i in &*t.ingredients
                        {
                            let resp = gloo_net::http::Request::post("http://127.0.0.1:8000/recipes/add_ingredient").mode(gloo_net::http::RequestMode::NoCors).json(&i).unwrap().send().await.unwrap();
                        }

                    });
                }
            ) { "Submit" }
            button(on:click = |_|
                {                 
                    let dialog_element: HtmlDialogElement = web_sys::window().unwrap().document().unwrap().get_element_by_id("confirm_dialog").unwrap().dyn_into().unwrap();
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
            form(id = "recipe_form") {
            label(for = "recipe_name") { "Recipe Name" }
            input(id = "recipe_name", bind:value = name_ref, placeholder = "recipe name", required = true)
            input(bind:value = catagory_ref, placeholder = "catagory name", required = true, list = "catagory_datalist")
            br{}
            textarea(class = "textareaElement", bind:value = information_str, placeholder = "information", contenteditable=true)
            br{}
            textarea(class = "textareaElement", bind:value = preparation_str, placeholder = "preparation", contenteditable=true)
            br{}
            input(bind:value = amount_str, required=true)
            input(bind:value = unit_ref, placeholder = "unit name", list="unit_datalist", required=true)
            IngredientListCompotent(recipe_name = &props.recipe.name, ingredients=&props.ingredients)
            br{}
            input(type="file", accept="image/*", multiple=true, bind:value = test)
            p { (test.get())}
            img(src=test.get())
            button(type = "button", on:click = move |_| 
            {
                
                let form: HtmlFormElement = web_sys::window().unwrap().document().unwrap().get_element_by_id("recipe_form").unwrap().dyn_into().unwrap();
                if form.report_validity()
                {
                    let dialog_element: HtmlDialogElement = web_sys::window().unwrap().document().unwrap().get_element_by_id("confirm_dialog").unwrap().dyn_into().unwrap();
                    dialog_element.show_modal();
                }
            }) { "Submit" }
            p { (temp.get())}
        }
    }
}
}

fn validate_form<'a, G: Html>(cx: Scope<'a>, data: &Long, ingredients: &Vec<Ingredient>, units: &Vec<Unit>, catagories: &Vec<Catagory>) -> View<G>
{
    view! {cx, }
}

#[component]
fn IngredientListCompotent<'a, G: Html>(cx: Scope<'a>, props: IngredientListProps<'a>) -> View<G> {
    view! { cx,
        div {
        ul(style = "list-style-type: none; padding: 0")
        {
            Indexed(
                iterable = &props.ingredients,
                view = |cx, x|
                {
                    let amount_str = create_signal(cx, x.amount.get().to_string());
                    let amount_ref = create_ref(cx, x.amount);
                    let amount_memo = create_effect(cx, ||
                        {
                            match amount_str.get().parse::<f32>()
                            {
                                Ok(float) => amount_ref.set(float),
                                Err(e) => amount_ref.set(1.)
                            };
                        });
                    let unit_ref = create_ref(cx, x.unit_name);
                    let name_ref = create_ref(cx, x.ingredient_name);
                    view! { cx,
                        li(style = "padding-bottom: 5px; display:flex;") {
                            input(bind:value = amount_str, style = "width: 25%", placeholder="Amount", required=true)
                            input(bind:value = unit_ref, style = "width: 25%", placeholder = "Unit", list="unit_datalist", required = true)
                            input(bind:value = name_ref, style = "width: 50%", placeholder = "Ingredient", list="ingredient_datalist", required = true)
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
        }
    }
}

#[derive(Prop)]
struct IngredientListProps<'a> 
{   
    recipe_name: &'a RcSignal<String>,
    ingredients: &'a RxVecNestedRx<IngredientAndAmount>,
}

#[engine_only_fn]
async fn get_form_build_state(_: StateGeneratorInfo<()>, _req: Request) -> Result<FormData, BlamedError<reqwest::Error>>
{
    let resp = reqwest::get("http://127.0.0.1:8000/catagories")
        .await?
        .text()
        .await?;
    let catagories: Vec<Catagory> = serde_json::from_str(&resp).unwrap();

    let resp = reqwest::get("http://127.0.0.1:8000/ingredients")
        .await?
        .text()
        .await?;
    let ingredients: Vec<Ingredient> = serde_json::from_str(&resp).unwrap();

    let resp = reqwest::get("http://127.0.0.1:8000/units")
        .await?
        .text()
        .await?;
    let units: Vec<Unit> = serde_json::from_str(&resp).unwrap();

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
        catagories
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
    ) -> IngredientAndAmountPerseusRxIntermediate {
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
