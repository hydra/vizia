mod helpers;
pub use helpers::*;
use vizia::prelude::*;
use crate::document::{Document, DocumentContainer};
use crate::tabbed_ui::{DocumentTab, HomeTab, TabKind};

mod document {
    use vizia::prelude::*;

    #[derive(Clone, Data)]
    pub struct Document {
        pub id: String,
        pub name: String,
    }

    #[derive(Clone, Lens, Default)]
    pub struct DocumentContent {
        pub content: Option<String>
    }

    impl DocumentContent {
        // TODO Ensure this is called every time the tab is selected
        pub fn load(&mut self, id: &String) {
            // TODO somehow load the document content, e.g. from file.
            self.content.replace(format!("content for {}", &id));
        }
    }

    #[derive(Clone, Data, Lens)]
    pub struct DocumentContainer {
        pub document: Document,
    }

    impl View for DocumentContainer {}

    impl DocumentContainer {
        pub fn new(cx: &mut Context, document: Document) -> Handle<Self> {
            Self {
                document
            }.build(cx, |cx| {
                VStack::new(cx, |cx| {
                    Label::new(cx, DocumentContent::content.map(move |maybe_content| {
                        maybe_content.clone().unwrap_or("Loading...".to_string())
                    }))
                        .text_align(TextAlign::Center);
                }).child_space(Stretch(1.0));
            })
        }
    }
}

mod tabbed_ui {
    use vizia::prelude::*;
    use crate::document::document_container_derived_lenses::document;
    use crate::document::DocumentContainer;

    #[derive(Clone, Data)]
    pub enum TabKind {
        Home(HomeTab),
        Document(DocumentTab),
    }

    #[derive(Clone, Data)]
    pub struct DocumentTab {
        pub container: DocumentContainer,
    }

    impl DocumentTab {
        pub fn build_tab(&self, name: String) -> TabPair {
            let tab = TabPair::new(
                move |cx| {
                    Label::new(cx, name.clone()).hoverable(false);
                    Element::new(cx).class("indicator");
                },
                |cx| {
                    ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
                        DocumentContainer::new(cx, self.container.document.clone());
                    })
                        .background_color(Color::rgb(0xdd, 0xdd, 0xdd))
                        .height(Percentage(100.0))
                        .width(Percentage(100.0));
                },
            );

            tab
        }
    }

    #[derive(Clone, Data)]
    pub struct HomeTab {}

    impl HomeTab {
        pub fn build_tab(&self, name: String) -> TabPair {
            let tab = TabPair::new(
                move |cx| {
                    Label::new(cx, name.clone()).hoverable(false);
                    Element::new(cx).class("indicator");
                },
                |cx| {
                    ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
                        VStack::new(cx, |cx|{
                            Label::new(cx, "🏠 Home")
                                .text_align(TextAlign::Center);
                        }).child_space(Stretch(1.0));
                    })
                        .background_color(Color::rgb(0xbb, 0xbb, 0xbb))
                        .height(Percentage(100.0))
                        .width(Percentage(100.0));
                },
            );

            tab
        }
    }

    impl TabKind {
        pub fn name(&self) -> String {
            match self {
                TabKind::Home(_) => { "Home".to_string() }
                TabKind::Document(document_tab) => { document_tab.container.document.name.clone() }
            }
        }

        pub fn build_tab(&self) -> TabPair {
            match self {
                TabKind::Home(tab) => tab.build_tab(self.name()),
                TabKind::Document(tab) => tab.build_tab(self.name()),
            }
        }
    }
}

#[derive(Lens, Default)]
pub struct AppData {
    tabs: Vec<TabKind>,
}

impl AppData {
    pub fn create_tabs(&mut self) {
        self.tabs.extend(vec![
            TabKind::Home( HomeTab {} ),
            TabKind::Document( DocumentTab {
                container: DocumentContainer {
                    document: Document { id: "document_1".to_string(), name: "Document 1".to_string() }
                }
            }),
            TabKind::Document( DocumentTab {
                container: DocumentContainer {
                    document: Document { id: "document_2".to_string(), name: "Document 2".to_string() }
                }
            }),
        ]);
    }
}

impl Model for AppData {

    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        println!("event: {:?}", &event);
        event.map(|app_event, _meta| match app_event {
            TabEvent::SetSelected(id) => {
                println!("id: {}", id);
            }
        });
    }
}

fn main() -> Result<(), ApplicationError> {

    Application::new(|cx| {

        let mut app_data = AppData::default();
        app_data.create_tabs();
        app_data.build(cx);

        ExamplePage::new(cx, |cx| {
            TabView::new(cx, AppData::tabs, |cx, tab_kind_lens| {
                tab_kind_lens.get(cx).build_tab()
            })
                .width(Percentage(100.0))
                .height(Percentage(100.0));
        });
    })
        .title("Tabview")
        .run()
}
