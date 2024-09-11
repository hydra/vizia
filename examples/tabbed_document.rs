mod helpers;

use log::trace;
pub use helpers::*;
use vizia::prelude::*;
use crate::document::Document;
use crate::route::Route;
use crate::tabbed_ui::{DocumentTab, HomeTab, TabKind};

mod document {
    use std::thread::sleep;
    use vizia::prelude::*;
    use crate::Route;

    #[derive(Debug)]
    enum DocumentEvent {
        Load { id: String },
        Loaded { content: DocumentContent }
    }

    #[derive(Debug)]
    pub enum DocumentRouteEvent {
        RouteChanged { document_id: String, route: Route }
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
                sleep(Duration::from_millis(250));

                let content = match id.as_str() {
                    "document_1" => DocumentContent {
                        content: Some("content for document 1".to_string()),
                        sections: vec![
                            "Section 4".to_string(),
                            "Section 2".to_string(),
                        ],
                    },
                    "document_2" => DocumentContent {
                        content: Some("content for document 1".to_string()),
                        sections: vec![
                            "Section 6".to_string(),
                            "Section 9".to_string(),
                        ],
                    },
                    _ => unreachable!()
                };
                let result = cp.emit(DocumentEvent::Loaded { content });
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

                        cx.emit(DocumentRouteEvent::RouteChanged { document_id: self.document.id.clone(), route: Route(Some(index)) })
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
        pub fn new(cx: &mut Context, document: Document, active_section: Option<usize>) -> Handle<Self> {
            let id = document.id.clone();

            Self {
                document,
                content: DocumentContent::default(),
                active_section,
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

mod route {
    use vizia::prelude::Data;

    #[derive(Clone, Debug, Data)]
    pub struct Route(pub Option<usize>);
}

mod tabbed_ui {
    use vizia::prelude::*;
    use crate::document::{Document, DocumentContainer, DocumentRouteEvent};
    use crate::route::Route;

    #[derive(Clone, Data)]
    pub enum TabKind {
        Home(HomeTab),
        Document(DocumentTab),
    }

    #[derive(Clone, Data)]
    pub struct DocumentTab {
        pub document: Document,
        pub route: Route,
    }

    impl DocumentTab {
        pub fn build_tab(&self) -> TabPair {
            let document = self.document.clone();
            let name = document.name.clone();
            let active_section = self.route.0.clone();

            let tab = TabPair::new(
                move |cx| {
                    Label::new(cx, name.clone()).hoverable(false);
                    Element::new(cx).class("indicator");
                },
                move |cx| {
                    let document_for_scrollview = document.clone();
                    ScrollView::new(cx, 0.0, 0.0, false, true, move |cx| {
                        DocumentContainer::new(cx, document_for_scrollview.clone(), active_section);
                    })
                        .background_color(Color::rgb(0xdd, 0xdd, 0xdd))
                        .height(Percentage(100.0))
                        .width(Percentage(100.0));
                },
            );

            tab
        }

        pub fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
            event.map(|event, _meta| {
                println!("section event: {:?}", &event);
                match event {
                    DocumentRouteEvent::RouteChanged { document_id, route } => {
                        if document_id.eq(&self.document.id) {
                            self.route = route.clone()
                        }
                    }
                }
            });
        }
    }

    #[derive(Clone, Data)]
    pub struct HomeTab {
        pub route: Route,
    }

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

        pub fn event(&mut self, _cx: &mut EventContext, _event: &mut Event) {
            // nothing to do
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

        pub fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
            match self {
                TabKind::Home(tab) => tab.event(cx, event),
                TabKind::Document(tab) => tab.event(cx, event)
            }
        }
    }
}

#[derive(Lens)]
pub struct MultiDocumentContainer {
    tabs: Vec<TabKind>,
}

impl View for MultiDocumentContainer {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        trace!("event: {:?}", &event);
        for tab in self.tabs.iter_mut() {
            tab.event(cx, event);
        }
    }
}

impl MultiDocumentContainer {
    pub fn new(cx: &mut Context, tabs: Vec<TabKind>) -> Handle<Self> {
        Self {
            tabs,
        }.build(cx, |cx| {
            TabView::new(cx, MultiDocumentContainer::tabs, |cx, tab_kind_lens| {
                tab_kind_lens.get(cx).build_tab()
            })
                .width(Percentage(100.0))
                .height(Percentage(100.0));
        })
    }
}

#[derive(Lens, Default)]
pub struct AppData {
}

impl AppData {
}

impl Model for AppData {

    fn event(&mut self, _cx: &mut EventContext, _event: &mut Event) {
    }
}

fn main() -> Result<(), ApplicationError> {

    Application::new(|cx| {

        let app_data = AppData::default();
        app_data.build(cx);

        ExamplePage::new(cx, |cx| {
            MultiDocumentContainer::new(cx, create_tabs())
                .width(Percentage(100.0))
                .height(Percentage(100.0));

        });
    })
        .title("Tabview")
        .run()
}

pub fn create_tabs() -> Vec<TabKind>{
    vec![
        TabKind::Home( HomeTab {
            route: Route(None)
        } ),
        TabKind::Document( DocumentTab {
            document: Document { id: "document_1".to_string(), name: "Document 1".to_string() },
            route: Route(None),
        }),
        TabKind::Document( DocumentTab {
            document: Document { id: "document_2".to_string(), name: "Document 2".to_string() },
            route: Route(None),
        }),
    ]
}

