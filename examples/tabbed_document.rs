mod helpers;
pub use helpers::*;
use vizia::prelude::*;
use crate::document::Document;
use crate::tabbed_ui::{DocumentTab, HomeTab, TabKind};

mod document {
    use std::thread::sleep;
    use vizia::prelude::*;

    enum DocumentEvent {
        Load { id: String },
        Loaded { content: String }
    }

    #[derive(Clone, Data, Lens)]
    pub struct Document {
        pub id: String,
        pub name: String,
    }

    #[derive(Clone, Lens, Default)]
    pub struct DocumentContent {
        pub content: Option<String>
    }

    impl DocumentContent {
        pub fn load(&mut self, cx: &mut EventContext, id: &String) {
            // Simulate loading a file, slowly.
            let id = id.clone();
            cx.spawn(move |cp|{
                sleep(Duration::from_secs(1));
                let content = format!("content for {}", id);
                cp.emit(DocumentEvent::Loaded { content }).expect("not broken");
            });
        }
    }

    #[derive(Clone, Lens)]
    pub struct DocumentContainer {
        pub document: Document,
        pub content: DocumentContent,
    }

    impl View for DocumentContainer {
        fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
            event.take(|event, meta| {
                match event {
                    DocumentEvent::Load { id } => {
                        self.content.load(cx, &id);
                    }
                    DocumentEvent::Loaded { content} => {
                        self.content.content.replace(content);
                    }
                }
            })
        }
    }

    impl DocumentContainer {
        pub fn new(cx: &mut Context, document: Document) -> Handle<Self> {
            let id = document.id.clone();

            Self {
                document,
                content: DocumentContent::default()
            }.build(cx, |cx| {
                VStack::new(cx, |cx| {

                    Label::new(cx, DocumentContainer::content.map(move |content| {
                        content.content.clone().unwrap_or("Loading...".to_string())
                    }))
                        .text_align(TextAlign::Center);
                }).child_space(Stretch(1.0));
            }).on_build(move |ecx| {
                ecx.emit(DocumentEvent::Load { id: id.clone() })
            })
        }
    }
}

mod tabbed_ui {
    use vizia::prelude::*;
    use crate::document::{Document, DocumentContainer};

    #[derive(Clone, Data)]
    pub enum TabKind {
        Home(HomeTab),
        Document(DocumentTab),
    }

    #[derive(Clone, Data)]
    pub struct DocumentTab {
        pub document: Document,
    }

    impl DocumentTab {
        pub fn build_tab(document: Document) -> TabPair {
            let name = document.name.clone();

            let tab = TabPair::new(
                move |cx| {
                    Label::new(cx, name.clone()).hoverable(false);
                    Element::new(cx).class("indicator");
                },
                move |cx| {
                    let document_for_scrollview = document.clone();
                    ScrollView::new(cx, 0.0, 0.0, false, true, move |cx| {
                        DocumentContainer::new(cx, document_for_scrollview.clone());
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
                TabKind::Document(document_tab) => { document_tab.document.name.clone() }
            }
        }

        pub fn build_tab(&self) -> TabPair {
            match self {
                TabKind::Home(tab) => tab.build_tab(self.name()),
                TabKind::Document(tab) => DocumentTab::build_tab(tab.document.clone()),
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
                document: Document { id: "document_1".to_string(), name: "Document 1".to_string() },
            }),
            TabKind::Document( DocumentTab {
                document: Document { id: "document_2".to_string(), name: "Document 2".to_string() },
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
