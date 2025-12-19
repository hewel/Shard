use iced::widget::{
    button, checkbox, column, container, row, scrollable, text, text_editor, text_input,
};
use iced::{Element, Length, Task, Theme};
use regex::Regex;

pub fn main() -> iced::Result {
    iced::application(Shard::new, Shard::update, Shard::view)
        .title("Shard")
        .theme(Shard::theme)
        .subscription(Shard::subscription)
        .run()
}

struct Shard {
    content: text_editor::Content,
    regex_pattern: String,
    replacement_text: String,
    snippets: Vec<Snippet>,
    is_listening_clipboard: bool,
    last_clipboard_content: Option<String>,
}

impl Default for Shard {
    fn default() -> Self {
        Self {
            content: text_editor::Content::new(),
            regex_pattern: String::new(),
            replacement_text: String::new(),
            snippets: Vec::new(),
            is_listening_clipboard: false,
            last_clipboard_content: None,
        }
    }
}

#[derive(Debug, Clone)]
struct Snippet {
    id: usize,
    title: String,
    content: String,
}

#[derive(Debug, Clone)]
enum Message {
    Editor(text_editor::Action),
    RegexPatternChanged(String),
    ReplacementTextChanged(String),
    ReplaceAll,
    RegexReplaceFinished(Result<String, String>),
    ToggleClipboard(bool),
    ClipboardTick,
    ClipboardContentReceived(Option<String>),
    PinSnippet,
    DeleteSnippet(usize),
    LoadSnippet(usize),
    CopySnippet(usize),
    CopyFinished,
}

impl Shard {
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Editor(action) => {
                self.content.perform(action);
                Task::none()
            }
            Message::RegexPatternChanged(pattern) => {
                self.regex_pattern = pattern;
                Task::none()
            }
            Message::ReplacementTextChanged(text) => {
                self.replacement_text = text;
                Task::none()
            }
            Message::ReplaceAll => {
                let pattern = self.regex_pattern.clone();
                let replacement = self.replacement_text.clone();
                let text = self.content.text();

                Task::perform(
                    async move {
                        match Regex::new(&pattern) {
                            Ok(re) => {
                                let new_text = re.replace_all(&text, &replacement).to_string();
                                Ok(new_text)
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::RegexReplaceFinished,
                )
            }
            Message::RegexReplaceFinished(result) => {
                match result {
                    Ok(new_text) => {
                        self.content = text_editor::Content::with_text(&new_text);
                    }
                    Err(error) => {
                        // TODO: Show error in status bar
                        eprintln!("Regex error: {}", error);
                    }
                }
                Task::none()
            }
            Message::ToggleClipboard(is_listening) => {
                self.is_listening_clipboard = is_listening;
                Task::none()
            }
            Message::ClipboardTick => Task::perform(
                async {
                    match arboard::Clipboard::new() {
                        Ok(mut clipboard) => clipboard.get_text().ok(),
                        Err(_) => None,
                    }
                },
                Message::ClipboardContentReceived,
            ),
            Message::ClipboardContentReceived(content) => {
                if let Some(text) = content {
                    if !text.is_empty() && Some(&text) != self.last_clipboard_content.as_ref() {
                        self.last_clipboard_content = Some(text.clone());
                        // Auto-pin
                        let id = self.snippets.len();
                        let title = text
                            .lines()
                            .next()
                            .unwrap_or("Untitled")
                            .chars()
                            .take(20)
                            .collect::<String>();
                        self.snippets.push(Snippet {
                            id,
                            title,
                            content: text,
                        });
                    }
                }
                Task::none()
            }
            Message::PinSnippet => {
                let content = self.content.text();
                if !content.is_empty() {
                    let id = self.snippets.len();
                    let title = content
                        .lines()
                        .next()
                        .unwrap_or("Untitled")
                        .chars()
                        .take(20)
                        .collect::<String>();
                    self.snippets.push(Snippet { id, title, content });
                }
                Task::none()
            }
            Message::DeleteSnippet(id) => {
                self.snippets.retain(|s| s.id != id);
                Task::none()
            }
            Message::LoadSnippet(id) => {
                if let Some(snippet) = self.snippets.iter().find(|s| s.id == id) {
                    self.content = text_editor::Content::with_text(&snippet.content);
                }
                Task::none()
            }
            Message::CopySnippet(id) => {
                if let Some(snippet) = self.snippets.iter().find(|s| s.id == id) {
                    let text = snippet.content.clone();
                    Task::perform(
                        async move {
                            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                let _ = clipboard.set_text(text);
                            }
                        },
                        |_| Message::CopyFinished,
                    )
                } else {
                    Task::none()
                }
            }
            Message::CopyFinished => Task::none(),
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        if self.is_listening_clipboard {
            iced::time::every(std::time::Duration::from_millis(1000))
                .map(|_| Message::ClipboardTick)
        } else {
            iced::Subscription::none()
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let toolbar = row![
            text_input("Regex Pattern", &self.regex_pattern).on_input(Message::RegexPatternChanged),
            text_input("Replacement", &self.replacement_text)
                .on_input(Message::ReplacementTextChanged),
            button("Replace All").on_press(Message::ReplaceAll),
            row![
                checkbox(self.is_listening_clipboard).on_toggle(Message::ToggleClipboard),
                text("Listen Clipboard"),
            ]
            .spacing(5),
        ]
        .spacing(10)
        .padding(10);

        let editor = text_editor(&self.content)
            .on_action(Message::Editor)
            .highlight("rs", iced::highlighter::Theme::SolarizedDark)
            .height(Length::Fill);

        let snippets_list = column(self.snippets.iter().map(|snippet| {
            row![
                text(&snippet.title),
                button("Load").on_press(Message::LoadSnippet(snippet.id)),
                button("Copy").on_press(Message::CopySnippet(snippet.id)),
                button("Del").on_press(Message::DeleteSnippet(snippet.id)),
            ]
            .spacing(5)
            .into()
        }))
        .spacing(10);

        let sidebar = column![
            text("Pinned Snippets").size(20),
            button("Pin Current").on_press(Message::PinSnippet),
            scrollable(snippets_list)
        ]
        .width(Length::FillPortion(1))
        .padding(10)
        .spacing(10);

        let main_area = row![
            container(editor).width(Length::FillPortion(3)).padding(10),
            container(sidebar)
                .width(Length::FillPortion(1))
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    container::Style::default().border(iced::Border {
                        color: palette.background.strong.color,
                        width: 1.0,
                        radius: 0.0.into(),
                    })
                }),
        ];

        let status_bar = row![
            text("Line: ?, Col: ?"), // TODO: Find cursor position API
            text(if self.is_listening_clipboard {
                "Listening to Clipboard"
            } else {
                "Ready"
            }),
        ]
        .spacing(20)
        .padding(5);

        column![toolbar, main_area, status_bar,].into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
