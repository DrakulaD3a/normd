use askama::Template;

#[derive(Template)]
#[template(path = "markdown.html")]
pub struct MarkdownTemplate {
    pub content: String,
}

#[derive(Template)]
#[template(path = "files.html")]
pub struct FilesTemplate {
    pub content: String,
}
