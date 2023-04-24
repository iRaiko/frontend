use sycamore::prelude::*;
use web_sys::{HtmlDialogElement, HtmlInputElement};
use perseus::{prelude::*, state::rx_collections::RxVecNestedRx};
use crate::templates::update_recipe::{ImageFilePerseusRxIntermediate, ImageFile};

#[component]
pub fn FileSelector<'a, G: Html>(cx: Scope<'a>, props: FileSelectorProps<'a>) -> View<G>
{
    let FileSelectorProps { recipe_name, images } = props;

    let current_index = create_signal(cx, images.get().len() as u32);

    let current_drag_index = create_signal(cx, 0);
    let target_drag_index = create_signal(cx, None);

    let backend_images: &Signal<Vec<ImagesSelected>> = create_signal(cx, Vec::new());

    #[cfg(client)]
    spawn_local_scoped(cx, async move 
    { 
        let response = gloo_net::http::Request::get("http://192.168.68.105:8000/images/names").mode(gloo_net::http::RequestMode::Cors).send().await;
        if let Ok(response_body) = response
        {
                let key = response_body.text().await.unwrap();
                let images: Vec<String> = serde_json::from_str(&key).unwrap();
                for i in images.into_iter()
                {
                    backend_images.modify().push(ImagesSelected { name: i, is_selected: create_signal(cx, false)} )
                }
        }
        else if let Err(e) = response
        {

        }
    });
    let file_input_ref = create_node_ref(cx);

    let view = create_memo(cx, move ||
    {
        let mut views = Vec::new();

        let mut image_list = (*images.get()).clone();
        image_list.sort_by_key(|key: &ImageFilePerseusRxIntermediate| key.index.get());

        for i in image_list.into_iter()
        {
            let target_drag_ref = i.index.clone();
            let index_dragstart_ref = i.index.clone();
            let index_dragover_ref = i.index.clone();

            let name_ref = create_ref(cx, i.image_name.clone());

            let url;

            let text_input;

            if i.data.get().is_some()
            {
                url = (*i.data.get()).clone().unwrap();
                text_input = view! { cx, input(bind:value = name_ref) };
            }
            else
            {
                let link_name = (*i.image_name.get()).clone();
                url = format!("http://192.168.68.105:8000/images/{}", link_name);  
                text_input = view! { cx, 
                            input(readonly = true, bind:value = name_ref)
                    };
            }
            views.push(
                {
                    view! { cx,                                 
                        div(class = "container",
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
                        })
                        { 
                            div(class = "img") 
                            {   
                                img(src=url) 
                            }
                            div(class = "input") 
                            {                                    
                                (text_input)
                            }
                            div(class = "button") 
                            {
                                button(type="button", on:click = move |_|
                                {
                                    let index = images.get().iter().position(|x| x.index.get() == i.index.get());
                                    if let Some(i) = index
                                    {
                                        images.modify().remove(i);
                                    }                                        
                                    images.modify().sort_by_key(|key| key.index.get());
                                    for (index, image) in images.get().iter().enumerate()
                                    {
                                        image.index.set(index as u32);
                                        current_index.set(index as u32 + 1);
                                    }
                                }) { "x" }
                            }
                            p { "" }
                        }
                    }
                }
            );
        }
        View::new_fragment(views.into_iter().collect())
    });

    let dialog_ref = create_node_ref(cx);

    let dialog = view! { cx, 
        dialog(ref = dialog_ref, id = "server_image_selector")
        {
            div
            {
                Indexed(
                    iterable = backend_images,
                    view = |cx, x|
                    {
                        (if true
                        {
                            let input_name = x.name.clone();
                            let label_name = x.name.clone();
                            let img_name = format!("http://192.168.68.105:8000/images/{}", x.name.clone());
                            let p_name = x.name.clone();
                            view! { cx,                     
                                input(type = "checkbox", bind:checked = x.is_selected, id = input_name)
                                label(class = "selector", for = label_name) 
                                { 
                                    img(src = img_name)
                                    p { (p_name) }
                                }
                            }
                        }
                        else
                        {
                            view! {cx, }
                        })
                    }
                )
            }
            button(type="button", on:click = move |_| 
            { 
                for i in backend_images.get().iter()
                {
                    if *i.is_selected.get()
                    {                              
                        let name_clone = i.name.clone();       
                        images.modify().push(
                            ImageFilePerseusRxIntermediate { recipe_name: create_rc_signal((*recipe_name.get()).clone()), image_name: create_rc_signal(name_clone), data: create_rc_signal(None), index: create_rc_signal(*current_index.get()) }
                        );
                        current_index.set(*current_index.get() + 1);
                        i.is_selected.set(false)
                    }
                }
                let hydrate_node = dialog_ref.get::<HydrateNode>();
                let dialog_element: HtmlDialogElement = hydrate_node.unchecked_into();
                dialog_element.close();
            }
            ) { "Confirm" }            
            button(type="button", on:click = |_| 
            {
                for i in backend_images.get().iter()
                {
                    i.is_selected.set(false)
                }
                let hydrate_node = dialog_ref.get::<HydrateNode>();
                let dialog_element: HtmlDialogElement = hydrate_node.unchecked_into();
                dialog_element.close();
            }
            ) { "cancel" }
        }
    };

    view! { cx, 
        label(class = "selector", for = "file_input") { "Select a file bruv" }
        input(
            id = "file_input",
            ref = file_input_ref, 
            type = "file",
            accept = "image/*",
            multiple = true,
            style = "opacity: 0;",
            on:change = move |_|
            {
                let hydrate_node = file_input_ref.get::<HydrateNode>();
                let input: HtmlInputElement = hydrate_node.unchecked_into();
                let files = input.files().unwrap(); // Would be null if input type is not file
                for i in 0..files.length()
                {
                    let url = web_sys::Url::create_object_url_with_blob(&files.get(i).unwrap()).unwrap();
                    images.modify().push(
                        ImageFilePerseusRxIntermediate
                        {
                            recipe_name: create_rc_signal((*recipe_name.get()).clone()),
                            image_name: create_rc_signal(files.get(i).unwrap().name()),
                            data: create_rc_signal(Some(url)),
                            index: create_rc_signal(*current_index.get()),
                        });
                    current_index.set(*current_index.get() + 1);
                }
            }
        )
        div(class = "preview")
        {
            (*view.get())
        }
            (dialog)
            button(type="button", on:click = |_| 
                {
                    let hydrate_node = dialog_ref.get::<HydrateNode>();
                    let dialog_element: HtmlDialogElement = hydrate_node.unchecked_into();
                    let _ = dialog_element.show_modal();
                }
            ) { "Select image from server" } 
        }
}

#[derive(Prop)]
pub struct FileSelectorProps<'a> 
{   
    recipe_name: &'a RcSignal<String>,
    images: &'a RxVecNestedRx<ImageFile>,
}

#[derive(Clone, PartialEq)]
struct ImagesSelected<'a>
{
    name: String,
    is_selected: &'a Signal<bool>
}