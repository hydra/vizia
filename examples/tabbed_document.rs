mod helpers;
pub use helpers::*;
use vizia::prelude::*;
use crate::document::Document;

mod document {
    use vizia_core::prelude::Data;

    #[derive(Clone, Data)]
    pub struct Document {
        pub id: String,
        pub name: String,
    }
}

#[derive(Lens, Default)]
pub struct AppData {
    documents: Vec<Document>,
}

impl AppData {
    pub fn load_documents(&mut self) {
        self.documents.extend(vec![
           Document { id: "document_1".to_string(), name: "Document 1".to_string() },
           Document { id: "document_2".to_string(), name: "Document 2".to_string() },
        ]);
    }
}

impl Model for AppData {}

fn main() -> Result<(), ApplicationError> {

    Application::new(|cx| {

        let mut app_data = AppData::default();
        app_data.load_documents();
        app_data.build(cx);

        ExamplePage::new(cx, |cx| {
            TabView::new(cx, AppData::documents, |cx, item| {
                let tab = TabPair::new(
                    move |cx| {
                        Label::new(cx, item.map(|document|document.name.clone())).hoverable(false);
                        Element::new(cx).class("indicator");
                    },
                    |cx| {
                        Element::new(cx).size(Percentage(100.0)).background_color(Color::rgb(0xdd, 0xdd, 0xdd));
                    },
                );

                tab
            })
                .width(Percentage(100.0))
                .height(Percentage(100.0));
        });
    })
        .title("Tabview")
        .run()
}
