use leptos::{html, prelude::*};
use leptos_router::hooks::use_query_map;
use crate::error::ContactFormError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactFormData {
    message: String,
    email: String,
    terms: Option<String>, // Checkboxes send "on" (or value) if checked, nothing if not.
}

#[island]
pub fn InteractiveContactForm(
    initial_success_message: Option<String>,
    initial_error: Option<ContactFormError>
) -> impl IntoView {
    // Form Submit Action
    let submit_action = ServerAction::<StoreContactForm>::new();
    let is_pending = move || submit_action.pending().get();

    // Signals for displaying success/error messages, initialized from props
    let displayed_success_message = RwSignal::new(initial_success_message.unwrap_or_default());
    let displayed_error = RwSignal::new(initial_error);

    // Form field states
    let message_rw = RwSignal::new(String::default());
    let email_rw = RwSignal::new(String::default());
    let terms_agreed_rw = RwSignal::new(false);
    let form_ref: NodeRef<html::Form> = NodeRef::new();
    let message_touched_rw = RwSignal::new(false); 
    let email_touched_rw = RwSignal::new(false);   

    // Client-side validation signals
    let message_is_empty = Signal::derive(move || message_rw.get().trim().is_empty());
    let email_is_invalid_format = Signal::derive(move || {
        let email_val = email_rw.get();
        email_val.is_empty() || !email_val.contains('@') 
    });
    let is_form_valid = RwSignal::new(true); 

    Effect::new(move |_| {
        is_form_valid.set( !message_is_empty.get() && !email_is_invalid_format.get() && terms_agreed_rw.get() );
    });

    // Show/hide client-side validation hints
    let show_message_hint = Signal::derive(move || message_touched_rw.get() && message_is_empty.get());
    let show_email_hint = Signal::derive(move || email_touched_rw.get() && email_is_invalid_format.get());

    // Effect to handle ServerAction results
    Effect::new(move |_| {
        if let Some(result) = submit_action.value().get() {
            match result {
                Ok(_) => {
                    // Though success often means redirect, this handles cases where action resolves successfully client-side.
                    displayed_success_message.set("Ditt meddelande är mottaget. Vi återkopplar snart.".to_string());
                    displayed_error.set(None);
                    // Clear form fields and touched states
                    message_rw.set(String::new());
                    email_rw.set(String::new());
                    terms_agreed_rw.set(false);
                    message_touched_rw.set(false);
                    email_touched_rw.set(false);
                }
                Err(e) => {
                    displayed_error.set(Some(e));
                    displayed_success_message.set(String::new());
                }
            }
        }
    });

    // Effect to clear success/error messages when user starts typing again
    Effect::new(move |prev_inputs: Option<(String, String, bool)>| {
        let current_inputs = (message_rw.get(), email_rw.get(), terms_agreed_rw.get());
        if let Some(previous) = prev_inputs { // Only run if not the first time (prev_inputs is populated)
            if previous != current_inputs { // Only run if inputs actually changed
                if displayed_error.get_untracked().is_some() {
                    displayed_error.set(None);
                }
            }
        }
        current_inputs // Return current_inputs to be used as prev_inputs in the next iteration
    });

    // Helper to check if the current displayed_error matches a specific field error variant.
    let is_field_error_variant = move |expected_variant_discriminant: std::mem::Discriminant<ContactFormError>| {
        displayed_error.get().map_or(false, |current_error| {
            std::mem::discriminant(&current_error) == expected_variant_discriminant
        })
    };
    
    // Signal to playwright to start
    Effect::new(move |_| {
        if let Some(el) = form_ref.get() {
            el.set_attribute("data-testid", "contact-form").expect("could not set attribute");
       }
    });

    view! {
        <ActionForm action=submit_action node_ref=form_ref>
            <Show when=move || !displayed_success_message.get().is_empty()>
                <p class="success-message">{move || displayed_success_message.get()}</p>
            </Show>
            // Display general server error if not field-specific
            <Show when=move || {
                displayed_error.get().map_or(false, |err| {
                    // Show general message for DatabaseError or if it's not any of the known field-specific errors
                    match err {
                        ContactFormError::DatabaseError(_) => true,
                        _ => !is_field_error_variant(std::mem::discriminant(&ContactFormError::MissingEmail)) &&
                             !is_field_error_variant(std::mem::discriminant(&ContactFormError::InvalidEmailFormat)) &&
                             !is_field_error_variant(std::mem::discriminant(&ContactFormError::EmailTooLong)) &&
                             !is_field_error_variant(std::mem::discriminant(&ContactFormError::MissingMessage)) &&
                             !is_field_error_variant(std::mem::discriminant(&ContactFormError::MessageTooLong)) &&
                             !is_field_error_variant(std::mem::discriminant(&ContactFormError::TermsNotAccepted))
                    }
                })
            }>
                <p class="error-message general-error">
                    { move || displayed_error.get().map(|e| e.get_user_message()).unwrap_or_default() }
                </p>
            </Show>

            <Show when=move || cfg!(debug_assertions)>
                <div>"Message Touched: " {move || message_touched_rw.get().to_string()}</div>
                <div>"Message Empty: " {move || message_is_empty.get().to_string()}</div>
                <div>"Email Touched: " {move || email_touched_rw.get().to_string()}</div>
                <div>"Email Invalid Format: " {move || email_is_invalid_format.get().to_string()}</div>
                <div>"Terms Agreed: " {move || terms_agreed_rw.get().to_string()}</div>
                <div>"Is Form Valid: " {move || is_form_valid.get().to_string()}</div>
                <div>"Displayed Error: " {move || format!("{:?}", displayed_error.get())}</div>
                <div>"Displayed Success: " {move || displayed_success_message.get()}</div>
            </Show>

            // Message Field
            <div class="form-field">
                <label for="message">"Skriv ett meddelande:"</label>
                <textarea
                    id="message"
                    name="message"
                    placeholder="Meddelande..."
                    disabled=is_pending
                    bind:value=message_rw
                    on:blur=move |_| message_touched_rw.set(true)
                    class:error=move || is_field_error_variant(std::mem::discriminant(&ContactFormError::MissingMessage)) || is_field_error_variant(std::mem::discriminant(&ContactFormError::MessageTooLong))
                    aria-invalid=move || (is_field_error_variant(std::mem::discriminant(&ContactFormError::MissingMessage)) || is_field_error_variant(std::mem::discriminant(&ContactFormError::MessageTooLong))).to_string()
                    aria-describedby="message-error"
                />
                // Server error
                <Show when=move || is_field_error_variant(std::mem::discriminant(&ContactFormError::MissingMessage)) || is_field_error_variant(std::mem::discriminant(&ContactFormError::MessageTooLong))>
                    <p class="error-message" id="message-error">
                       {move || displayed_error.get().map(|e| e.get_user_message()).unwrap_or_default()}
                    </p>
                </Show>
                // Client error
                <Show when=move || show_message_hint.get()>
                     <p class="hint-message">"Meddelandet får inte vara tomt."</p>
                </Show>
            </div>

            // Email Field
            <div class="form-field">
                <label for="email">"Ange din email:"</label>
                <input
                    id="email"
                    type="email"
                    name="email"
                    placeholder="Din email..."
                    disabled=is_pending
                    bind:value=email_rw
                    on:blur=move |_| email_touched_rw.set(true)
                    class:error=move || is_field_error_variant(std::mem::discriminant(&ContactFormError::InvalidEmailFormat)) || is_field_error_variant(std::mem::discriminant(&ContactFormError::MissingEmail)) || is_field_error_variant(std::mem::discriminant(&ContactFormError::EmailTooLong))
                    aria-invalid=move || (is_field_error_variant(std::mem::discriminant(&ContactFormError::InvalidEmailFormat)) || is_field_error_variant(std::mem::discriminant(&ContactFormError::MissingEmail)) || is_field_error_variant(std::mem::discriminant(&ContactFormError::EmailTooLong))).to_string()
                    aria-describedby="email-error"
                />
                // Server error
                 <Show when=move || is_field_error_variant(std::mem::discriminant(&ContactFormError::InvalidEmailFormat)) || is_field_error_variant(std::mem::discriminant(&ContactFormError::MissingEmail)) || is_field_error_variant(std::mem::discriminant(&ContactFormError::EmailTooLong))>
                    <p class="error-message" id="email-error">
                        { move || displayed_error.get().map(|e| e.get_user_message()).unwrap_or_default() }
                    </p>
                </Show>
                // Client error
                <Show when=move || show_email_hint.get()>
                     <p class="hint-message">"Ange en giltig email."</p>
                </Show>
            </div>

             // Terms Checkbox
            <div class="form-field terms">
                <input
                    id="terms"
                    type="checkbox"
                    name="terms" 
                    disabled=is_pending
                    bind:checked=terms_agreed_rw
                    class:error=move || is_field_error_variant(std::mem::discriminant(&ContactFormError::TermsNotAccepted))
                    aria-invalid=move || is_field_error_variant(std::mem::discriminant(&ContactFormError::TermsNotAccepted)).to_string()
                    aria-describedby="terms-error"
                />
                <label for="terms">"Jag accepterar att informationen sparas. Uppgifterna tas bort efter slutfört ärende."</label> 
                 <Show when=move || is_field_error_variant(std::mem::discriminant(&ContactFormError::TermsNotAccepted))>
                    <p class="error-message" id="terms-error">
                       {move || displayed_error.get().map(|e| e.get_user_message()).unwrap_or_default()}
                    </p>
                </Show>
            </div>

            <input
                type="submit"
                data-testid="contact-form-submit"
                value=move ||  if is_pending() { "Skickar..." } else { "Skicka" } 
                disabled=move || is_pending() || !is_form_valid.get()
             />
        </ActionForm>
     }
}

#[component]
pub fn ContactForm() -> impl IntoView {
    let query_map = use_query_map();

    let initial_success_message = query_map.with(|params| {
        params.get("status").and_then(|s| if s == "success" { Some("Ditt meddelande är mottaget. Vi återkopplar snart.".to_string()) } else { None })
    });

    let initial_error = query_map.with(|params| {
        params.get("error").and_then(|error_str| ContactFormError::from_str(&error_str).ok())
    });

    view! {
        <InteractiveContactForm initial_success_message=initial_success_message initial_error=initial_error />
    }
}

#[server(StoreContactForm, "/api")]
pub async fn store_contact_form(
    message: String,
    email: String,
    terms: Option<String>,
) -> Result<(), ContactFormError> {
    use leptos_axum::{redirect, extract};
    use axum::http::HeaderMap;
    use leptos::logging::log;
        
    // --- Debug / Testing ---
    // Delay to make e2e testing stable.
    #[cfg(debug_assertions)]
    {
        use tokio::time::{sleep, Duration};
        sleep(Duration::from_millis(500)).await;
    }

    
    // --- Validation ---
    let email_trimmed = email.trim();
    let message_trimmed = message.trim();

    if email_trimmed.is_empty() {
        log!("Validation failed: MissingEmail");
        let error = ContactFormError::MissingEmail;
        redirect(&format!("/?error={}#contact", error.to_string()));
        return Err(error);
    }
    if message_trimmed.is_empty() {
        log!("Validation failed: MissingMessage");
        let error = ContactFormError::MissingMessage;
        redirect(&format!("/?error={}#contact", error.to_string()));
        return Err(error);
    }
    if terms.is_none() { // "on" if checked, None if not.
        log!("Validation failed: TermsNotAccepted");
        let error = ContactFormError::TermsNotAccepted;
        redirect(&format!("/?error={}#contact", error.to_string()));
        return Err(error);
    }
    if email_trimmed.len() > 254 {
        log!("Validation failed: EmailTooLong");
        let error = ContactFormError::EmailTooLong;
        redirect(&format!("/?error={}#contact", error.to_string()));
        return Err(error);
    }
    if message_trimmed.len() > 5000 {
        log!("Validation failed: MessageTooLong");
        let error = ContactFormError::MessageTooLong;
        redirect(&format!("/?error={}#contact", error.to_string()));
        return Err(error);
    }
    if !email_trimmed.contains('@')
        || email_trimmed.starts_with('@')
        || email_trimmed.ends_with('@')
    {
        log!("Validation failed: InvalidEmailFormat");
        let error = ContactFormError::InvalidEmailFormat;
        redirect(&format!("/?error={}#contact", error.to_string()));
        return Err(error);
    }

    // --- Database Operation ---
    // Guard DB access to be server-side only
    #[cfg(feature = "ssr")]
    {
        use crate::app::ssr::db;

        // Extract request HTTP-headers
        let headers: HeaderMap = extract().await.map_err(ServerFnError::from)?;        

        // Check if "Accept"-header indicate this is a JSON-request (from a reactive client)
        let accepts_json = headers
            .get("Accept")
            .and_then(|v| v.to_str().ok())
            .map_or(false, |s| s.contains("application/json"));

        let mut conn = match db().await {
            Ok(connection) => connection,
            Err(_) => {
                redirect("/?error=DatabaseError#contact");
                return Err(ContactFormError::DatabaseError("Kunde inte ansluta till databasen.".to_string()));
            }
        };

        match sqlx::query("INSERT INTO emails (email, message) VALUES ($1, $2)")
            .bind(email_trimmed)
            .bind(message_trimmed)
            .execute(&mut conn)
            .await
        {
            Ok(_result) => {
                log!("Successfully inserted contact form data into DB.");

                // If request is not from a reactive client (JS disabled in browser), redirect
                if !accepts_json {
                    redirect("/?status=success#contact");
                }
                
                Ok(())
            }
            Err(e) => {
                log!("Database execution error: {:?}", e);
                redirect("/?error=DatabaseError#contact");
                Err(ContactFormError::from(e))
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        // This block should ideally not be reached if store_contact_form is only called on SSR.
        // If it can be called from WASM directly (without SSR), this is a client-side mock/error.
        log!("store_contact_form called on client without SSR feature, this is not supported for DB operations.");
        redirect("/?error=DatabaseError#contact"); // Generic error
        Err(ServerFnError::ServerError("Funktionen stöds inte på klientsidan.".to_string()))
    }
}
