use serde::{Serialize, Deserialize};
use sycamore::prelude::*;
use crate::AppStateRx;
use perseus::prelude::*;

#[component]
pub fn Login<'a, G: Html>(cx: Scope<'a>, LoginProps { state, children }: LoginProps<'a, G> ) -> View<G>
{
    #[cfg(client)]
    spawn_local_scoped(cx, async move 
    {
        state.verify_key().await;
    });

    let child = children.call(cx);
    let children = child.clone();

    let password = create_signal(cx, "".to_string());
    let log_in_result = create_signal(cx, "".to_string());
    view! { cx,
        (if !*state.state.get()
        {
            view! {cx, 
                p { "you need to login bruv" }
                input(type="password", bind:value = password, placeholder = "Password")
                div {
                button(on:click = move |_| 
                    {
                        #[cfg(client)]
                        spawn_local_scoped(cx, async move 
                        { 
                            let login = LoginInformation { username: "Heuts".to_string(), password: (*password.get()).clone() };
                            let response = gloo_net::http::Request::post("http://192.168.68.100:8000/login/").mode(gloo_net::http::RequestMode::Cors).json(&login).unwrap().send().await;
                            if let Ok(response_body) = response
                            {
                                if response_body.ok()
                                {
                                    let key = response_body.text().await.unwrap();
                                    state.session_key.set(Some(key));                            
                                    state.verify_key().await;
                                }
                                else
                                {
                                    let result = response_body.text().await.unwrap();
                                    log_in_result.set(result);
                                }
                            }
                            else if let Err(e) = response
                            {
                                log_in_result.set(format!("{}", e));
                            }
                        }
                        );
                    }) { "Log in"}
                    p { (format!("{}", log_in_result.get())) }
                }
            }
        }
        else
        {
            let children = children.clone();
            view! { cx, (children) }
        })
    }
}

#[derive(Prop)]
pub struct LoginProps<'a, G: Html>
{
    pub state: &'a AppStateRx,
    pub children: Children<'a, G>
}

#[derive(Serialize, Deserialize)]
struct LoginInformation
{
    username: String,
    password: String,
}