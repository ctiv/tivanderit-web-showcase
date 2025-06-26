use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::contact::ContactForm;

#[cfg(feature="ssr")]
pub mod ssr {
    use leptos::server_fn::ServerFnError;
    use sqlx::{migrate::MigrateDatabase, Connection, SqliteConnection};

    pub async fn db() -> Result<SqliteConnection, ServerFnError> {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set - aborting startup because the database is required");
        let path = db_url.strip_prefix("sqlite:").unwrap_or(&db_url);

        if !sqlx::Sqlite::database_exists(path).await? {
            sqlx::Sqlite::create_database(path).await?;
        }
                
        Ok(SqliteConnection::connect(&db_url).await?)
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options islands=true/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/tivanderit.css"/>

        <Title text="Tivander IT"/>

        <Router>
            <NavBar/>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("") view=HomePage/>
                </Routes>
            </main>
            <Footer/>
        </Router>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <div class="footer">
            // Add the logo image with a class for styling
            <img class="footer-logo" src="/TivanderIT.png" alt="Tivander IT Logo" />

            // Wrap the text elements in a div for layout
            <div class="footer-text">
                <div><a href="mailto:hej@tivanderit.se">"✉️ hej@tivanderit.se"</a></div>
                <div>"Copyright © 2025 - Tivander IT AB"</div>
            </div>
        </div>
    }
}

// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div id="home">
            <h1>"Omvandlar idéer till digital verklighet"</h1>
            <p>"Vi bygger robusta, säkra och lättunderhållna mjukvarulösningar för alla. Oavsett om det gäller mindre appar eller komplexa backend-system, skapar vi eleganta och effektiva lösningar, anpassade efter dina behov och önskemål."</p>
        </div>
        <div id="about">
            <h2>"Vad vi erbjuder"</h2>
            <div id="development" class="card"> 
                <div class="card-image"/>
                <div class="card-text">
                    <h3>"Mjukvaruutveckling"</h3>
                    <p>"Vi jobbar med att omvandla idéer till digital verklighet och vi älskar det! Oavsett om du är ute efter en webbapplikation, ett automatiserat verktyg eller en mjukvaruuppgradering, har vi expertisen att leverera kvalitetskod som är:"</p>
                    <ul>
                        <li><b>"Stabil:"</b>" Designad för att hålla."</li>
                        <li><b>"Säker:"</b>" Skyddad mot sårbarheter."</li>
                        <li><b>"Lättunderhållen:"</b>" Enkel att utöka och förbättra."</li>
                    </ul>
                </div>
            </div>
            <div id="technician" class="card"> 
                <div class="card-image"/>
                <div class="card-text">
                    <h3>"Hjälp med persondatorer"</h3>
                    <p>"Har du problem med datorn? Vi kan hjälpa dig att rensa upp persondatorn, fixa allt från små till större fel och få allt att fungera smidigt igen. Om din dator krånglar eller börjar kännas långsam, finns vi här för att hjälpa till."</p>
                </div>
            </div>
        </div>
        <div id="contact">
            <div class="contact-info">
                <h2>"Kontakta oss"</h2>
                <p>"Låt oss skapa något fantastiskt tillsammans! Kontakta oss idag så hjälper vi dig med ditt nästa projekt eller med att få din dator på rätt spår igen."</p>
                <p>"Och du! Vi gillar det vi håller på med, därför kostar det inte skjortan att anlita oss. Vi levererar bra resultat till konkurrensmässiga priser. Hör av dig, så tar vi fram en offert."</p>
            </div>
            <ContactForm/>
        </div>
    }
}

#[component]
fn NavBar() -> impl IntoView {
    view! {
        <nav>
            <a href="#"><img src="/TivanderIT.png" alt="Tivander IT Logo" />
                <Show when=|| {cfg!(debug_assertions)}>
                    <div class="dev-mode">"Development mode"</div> 
                </Show>
            </a>
            <a href="#home">"Vårt uppdrag"</a>
            <a href="#about">"Vad vi erbjuder"</a>
            <a href="#contact">"Kontakta oss"</a>
        </nav>
    }
}
