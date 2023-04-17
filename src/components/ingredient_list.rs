use perseus::state::rx_collections::RxVecNestedRx;
use sycamore::prelude::*;

use crate::common::{IngredientAndAmount, IngredientAndAmountPerseusRxIntermediate};

#[component]
pub fn IngredientListCompotent<'a, G: Html>(cx: Scope<'a>, props: IngredientListProps<'a>) -> View<G> {
    let current_drag_index = create_signal(cx, 0);
    let target_drag_index = create_signal(cx, None);

    let view = create_memo(cx, move ||
    {
        let mut views = Vec::new();

        let mut ingredient_list = (*props.ingredients.get()).clone();
        ingredient_list.sort_by_key(|key| key.index.get());

        for i in ingredient_list.into_iter()
        {
            let starting_amount = i.amount.get().to_string();
            let amount_str = create_signal(cx, starting_amount);            
            let amount_ref = create_ref(cx, i.amount);
            create_effect(cx, 
                || 
                { 
                    match amount_str.get().parse::<f32>() 
                    { 
                        Ok(float) => amount_ref.set_silent(float), 
                        Err(_) => (), 
                    }; 
                }
            );
            let unit_ref = create_ref(cx, i.unit_name);
            let name_ref = create_ref(cx, i.ingredient_name);


            let target_drag_ref = i.index.clone();
            let index_dragstart_ref = i.index.clone();
            let index_dragover_ref = i.index.clone();

            views.push(
                view! { cx, 
                    li
                    (
                        draggable = true,
                        on:dragstart = move |_|
                        {
                            current_drag_index.set(*index_dragstart_ref.get());
                            target_drag_index.set(Some(target_drag_ref.clone()));
                        },
                        on:dragover = move |_|
                        {
                            if let Some(inner) = &*target_drag_index.get()
                            {
                                if *inner.get() != *index_dragover_ref.get()
                                {
                                    inner.set(*index_dragover_ref.get());
                                    let temporary_value = *current_drag_index.get();
                                    current_drag_index.set(*index_dragover_ref.get());
                                    index_dragover_ref.set(temporary_value);
                                }
                            }
                        },
                        on:dragend = move |_|
                        {
                            target_drag_index.set(None);
                        },
                        style = "padding-bottom: 5px; display:flex;") 
                        {             
                            div(class = "ingredient-div") {
                                input(bind:value = amount_str, placeholder = "Amount", pattern = "([0-9]*[.])?[0-9]+", required = true)
                            }
                            div(class = "ingredient-div") {
                                input(bind:value = unit_ref, placeholder = "Unit", list="unit_datalist", required = true)
                            }
                            div(class = "ingredient-div") {
                                input(bind:value = name_ref, placeholder = "Ingredient", list="ingredient_datalist", required = true)
                            }
                        }
                }
            )
        }
        View::new_fragment(views.into_iter().collect())
    });

    view! { cx,
        div(class = "label-ingredient-div") {
            label(for = "information_text_area") { "Amount" }
            label(for = "information_text_area") { "Unit" }
            label(for = "information_text_area") { "Name" }
        }
        ul(style = "list-style-type: none; padding: 0")
        {
            (*view.get())
        }
        button(type="button", on:click=
            {
                move |_|
                {
                    let ingredient = IngredientAndAmountPerseusRxIntermediate::new(props.recipe_name.clone(), "", props.ingredients.get().len() as u32, 1.0, "");
                    props.ingredients.modify().push(ingredient);
                }
            }) { "Add Ingredient" }        
        button(type="button", on:click=
            {
                move |_|
                {
                    let mut index = None;
                    let mut max = 0;
                    for i in 0..props.ingredients.get().len()
                    {
                        if *props.ingredients.get()[i].index.get() > max
                        {
                            max = *props.ingredients.get()[i].index.get();
                            index = Some(i);
                        }
                    }
                    if index.is_some()
                    {
                        props.ingredients.modify().remove(index.unwrap());
                    }
                }
            }) { "Remove Ingredient" }
        }
    }

#[derive(Prop)]
pub struct IngredientListProps<'a> 
{   
    recipe_name: &'a RcSignal<String>,
    ingredients: &'a RxVecNestedRx<IngredientAndAmount>,
}