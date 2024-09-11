mod helpers;

use log::trace;
pub use helpers::*;
use vizia::prelude::*;
use crate::document::Document;
use crate::tabbed_ui::{DocumentTab, HomeTab, TabKind};

mod document {
    use std::thread::sleep;
    use vizia::prelude::*;

    #[derive(Debug)]
    enum DocumentEvent {
        Load { id: String },
        Loaded { content: DocumentContent }
    }

    #[derive(Clone, Data, Lens)]
    pub struct Document {
        pub id: String,
        pub name: String,
    }

    #[derive(Clone, Lens, Default, Debug)]
    pub struct DocumentContent {
        pub content: Option<String>,
        pub sections: Vec<String>,
    }

    impl DocumentContent {
        pub fn load(&mut self, cx: &mut EventContext, id: &String) {
            // Simulate loading a file, slowly.
            let id = id.clone();
            cx.spawn(move |cp|{
                sleep(Duration::from_secs(1));
                let content = format!("content for {}", id);
                let document_content = DocumentContent {
                    content: Some(content),
                    sections: vec![
                        "Section 1".to_string(),
                        "Section 2".to_string(),
                    ],
                };
                let result = cp.emit(DocumentEvent::Loaded { content: document_content });
                match result {
                    Ok(_) => println!("emitted content, id: {}", id),
                    Err(e) => println!("failed to emit content, id: {}, error: {}", id, e),
                }
            });
        }
    }

    #[derive(Clone, Lens)]
    pub struct DocumentContainer {
        pub document: Document,
        pub content: DocumentContent,
        pub active_section: Option<usize>,
    }

    impl View for DocumentContainer {
        fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
            event.take(|event, _meta| {
                println!("section event: {:?}", &event);
                match event {
                    SectionEvent::Change { index } => {
                        self.active_section.replace(index);
                    }
                }
            });
            event.take(|event, _meta| {
                println!("document event: {:?}", &event);
                match event {
                    DocumentEvent::Load { id } => {
                        self.content.load(cx, &id);
                    }
                    DocumentEvent::Loaded { content} => {
                        self.content = content;
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
                content: DocumentContent::default(),
                active_section: None,
            }.build(cx, |cx| {

                HStack::new(cx, | cx | {
                    //
                    // Left
                    //
                    VStack::new(cx, |cx| {
                        let sections_lens = DocumentContainer::content.then(DocumentContent::sections);

                        List::new(cx, sections_lens, |cx, index, item| {

                            let foo = DocumentContainer::active_section.map(move |selection|{
                                let selected = match selection {
                                    Some(active_index) if *active_index == index => true,
                                    _ => false
                                };

                                println!("index: {}, selected: {}", index, selected);
                                selected
                            });

                            Label::new(cx, item).hoverable(false)
                                .background_color(foo.map(|foobar| match *foobar {
                                    true => Color::rgb(0x00, 0x00, 0xff),
                                    false => Color::rgb(0xdd, 0xdd, 0xdd),
                                }))
                                .width(Stretch(1.0))
                                .height(Pixels(30.0))
                                .checked(foo)
                                .on_press(move |ecx|ecx.emit(SectionEvent::Change { index }));
                        })
                            .child_space(Pixels(4.0));
                    })
                        .width(Pixels(200.0))
                        .height(Percentage(100.0));

                    //
                    // Divider
                    //
                    Element::new(cx)
                        .width(Pixels(2.0))
                        .height(Percentage(100.0))
                        .background_color(Color::gray());

                    //
                    // Right
                    //
                    VStack::new(cx, |cx| {

                        Label::new(cx, DocumentContainer::content.map(move |content| {
                            content.content.clone().unwrap_or("Loading...".to_string())
                        }))
                            .text_align(TextAlign::Center);
                    })
                        .child_space(Stretch(1.0));

                });

            }).on_build(move |ecx| {
                ecx.emit(DocumentEvent::Load { id: id.clone() })
            })
        }
    }

    #[derive(Debug)]
    enum SectionEvent {
        Change { index: usize }
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
        pub fn build_tab(&self) -> TabPair {
            let document = self.document.clone();
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
                TabKind::Document(tab) => tab.build_tab(),
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
        trace!("event: {:?}", &event);
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
