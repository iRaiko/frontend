use std::borrow::Borrow;

use sycamore::prelude::*;
use perseus::prelude::*;
use serde::{Serialize, Deserialize};
use web_sys::HtmlDialogElement;
use crate::{components::{layout::Layout, ingredient_list::IngredientListCompotent}, common::RecipeInformationPerseusRxIntermediate};

#[component]
pub fn FormCapsule<'a, G: Html>(cx: Scope<'a>, props: FormProps<'a>) -> View<G> 
{
    let recipe_name = props.information.get().recipe.name.clone();
    let ingredients = props.information.get().ingredients.clone();
    let amount_str = create_signal(cx, props.information.get().recipe.base_amount.get().to_string());
    let amount_ref = create_ref(cx, &props.information.get().recipe.base_amount.clone());
    create_effect(cx, || {
        match amount_str.get().parse::<f32>() {
            Ok(float) => amount_ref.set(float),
            Err(_) => amount_ref.set(1.),
        };
    });
    let information_str = create_signal(cx,         
        {
        if let Some(info) = props.information.get().recipe.information.get().borrow()
        {
            info.to_owned()
        }
        else
        {
            "".to_string()
        }
    });
    create_effect(cx, || {
        props.information.get()
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
            if let Some(info) = props.information.get().recipe.preparation.get().borrow()
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
        props.information.get()
            .recipe
            .preparation
            .set(if !preparation_str.get().is_empty() {
                Some(preparation_str.get().to_string())
            } else {
                None
            });
    });

    view! {cx,
            (create_datalist(cx, Datalist { id: "unit_datalist", iterable: &props.units } ))
            (create_datalist(cx, Datalist { id: "ingredient_datalist", iterable: &props.ingredients }))
            (create_datalist(cx, Datalist { id: "catagory_datalist", iterable: &props.catagories }))
            div(class = "form") {
            form(id = "recipe_form") {
                div(class = "form-row") {
                    div(class = "form-block") {
                        label(for = "recipe_name") { "Recipe Name" }
                        input(id = "recipe_name", bind:value = &props.information.get().recipe.name, placeholder = "recipe name", required = true, readonly = true)
                    }
                    div(class = "form-block") {
                        label(for = "catagory_name") { "Catagory Name" }
                        input(id = "catagory_name", bind:value = &props.information.get().recipe.catagory_name, placeholder = "catagory name", required = true, list = "catagory_datalist")
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
                        input(id = "recipe_unit_input", bind:value = &props.information.get().recipe.unit_name, placeholder = "unit name", list="unit_datalist", required=true)
                    }
                }
                div(class = "form-row") {
                    div(class = "form-block") 
                    {
                        IngredientListCompotent(recipe_name = &recipe_name, ingredients=&ingredients)
                    }
                }
                div(class = "form-row") {
                    input(type="file")
                    div(id = "preview"){}
                }
            }
        }
    }
}

#[derive(Prop)]
struct Datalist<'a>
{
    id: &'a str,
    iterable: &'a RcSignal<Vec<FINDGOODNAMEFORSTRUCT>>
}

#[component]
fn create_datalist<'a, G: Html>(cx: Scope<'a>, prop: Datalist<'a>) -> View<G>
{
    view! { cx, 
        datalist(id=prop.id)
        { Indexed(
            iterable=prop.iterable,
            view = |cx, x| view! {cx, 
                option(value = x.name)
            }
        )} 
    }
}


#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct FINDGOODNAMEFORSTRUCT
{
    pub name: String
}

#[derive(Prop)]
pub struct FormProps<'a> 
{   
    information: &'a RcSignal<RecipeInformationPerseusRxIntermediate>,     
    units: &'a RcSignal<Vec<FINDGOODNAMEFORSTRUCT>>,
    ingredients: &'a RcSignal<Vec<FINDGOODNAMEFORSTRUCT>>,
    catagories: &'a RcSignal<Vec<FINDGOODNAMEFORSTRUCT>>,
}
