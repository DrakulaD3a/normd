use askama::Template;
use std::collections::HashMap;
use std::fs::{read_dir, read_to_string};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

mod templates;

pub struct Server {
    port: u16,

    files: Vec<PathBuf>,
    markdown: HashMap<String, String>,
}

impl Server {
    /// Function to create a new `Server` instance.
    ///
    /// # Errors
    ///
    /// This function will return an error if the directory passed in can't be read or the files
    /// inside are not a valid markdown.
    pub fn new(port: u16, directory: &Path) -> anyhow::Result<Self> {
        let files = get_files(directory)?;
        let markdown: HashMap<String, String> = files
            .iter()
            .map(|f| {
                Ok((
                    f.display().to_string(),
                    markdown::to_html(&read_to_string(directory.join(f))?),
                ))
            })
            .collect::<anyhow::Result<HashMap<String, String>>>()?;

        Ok(Self {
            port,
            files,
            markdown,
        })
    }

    /// Function to start the server.
    ///
    /// # Errors
    ///
    /// This function will return an error if the server can't be started.
    pub async fn serve(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port)).await?;

        loop {
            self.handle_stream(listener.accept().await?.0).await?;
        }
    }

    async fn handle_stream(&self, mut stream: TcpStream) -> anyhow::Result<()> {
        let request_line = BufReader::new(&mut stream)
            .lines()
            .next_line()
            .await?
            .expect("No request line");
        let current_path =
            request_line.split_whitespace().collect::<Vec<_>>()[1].trim_start_matches('/');

        let files_to_show = self
            .files
            .iter()
            .filter(|f| f.strip_prefix(current_path).is_ok())
            .collect::<Vec<_>>();

        let response = if files_to_show.len() == 1 {
            let render: Option<String> = files_to_show
                .iter()
                .map(|f| Some(self.markdown.get(f.to_str()?)?.to_string()))
                .collect();

            let template = templates::MarkdownTemplate {
                content: render.expect("Couldn't parse file"),
            };
            format!("HTTP/1.1 200 OK\r\n\r\n{}", template.render()?)
        } else {
            let render = files_to_show.iter().fold(String::new(), |acc, f| {
                format!("{acc}<a href='{f}'>{f}</a><br>", f = f.display())
            });

            let template = templates::FilesTemplate { content: render };
            format!("HTTP/1.1 200 OK\r\n\r\n{}", template.render()?)
        };

        stream.write_all(response.as_bytes()).await?;

        stream.shutdown().await?;

        Ok(())
    }
}

fn get_files(directory: &Path) -> anyhow::Result<Vec<PathBuf>> {
    Ok(read_dir(directory)?
        .map(|e| {
            let path = e?.path();
            if path.is_dir() {
                Ok(get_files(&path)?)
            } else {
                Ok(vec![path.strip_prefix(directory)?.to_path_buf()])
            }
        })
        .collect::<anyhow::Result<Vec<Vec<PathBuf>>>>()?
        .iter()
        .flatten()
        .cloned()
        .collect())
}
