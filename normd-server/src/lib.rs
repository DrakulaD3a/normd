use askama::Template;
use std::collections::HashMap;
use std::fs::{read_dir, read_to_string};
use std::io;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

mod templates;

pub struct Server {
    port: u16,

    files: Vec<PathBuf>,
    markdown: HashMap<String, String>,
}

impl Server {
    #[must_use]
    pub fn new(port: u16, directory: &PathBuf) -> Self {
        let files = get_files(directory);
        let markdown: HashMap<String, String> = files
            .iter()
            .map(|f| {
                (
                    f.display().to_string(),
                    markdown::to_html(&read_to_string(directory.join(f)).unwrap()),
                )
            })
            .collect();

        Self {
            port,
            files,
            markdown,
        }
    }

    pub async fn serve(self) -> io::Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port)).await?;

        loop {
            self.handle_stream(listener.accept().await?.0).await?;
        }
    }

    async fn handle_stream(&self, mut stream: TcpStream) -> io::Result<()> {
        let request_line = BufReader::new(&mut stream)
            .lines()
            .next_line()
            .await?
            .unwrap();
        let current_path =
            request_line.split_whitespace().collect::<Vec<_>>()[1].trim_start_matches('/');

        let files_to_show = self
            .files
            .iter()
            .filter(|f| f.strip_prefix(current_path).is_ok())
            .collect::<Vec<_>>();

        let response = if files_to_show.len() == 1 {
            let render = files_to_show
                .iter()
                .map(|f| self.markdown.get(f.to_str().unwrap()).unwrap().to_string())
                .collect();

            let template = templates::MarkdownTemplate { content: render };
            format!("HTTP/1.1 200 OK\r\n\r\n{}", template.render().unwrap())
        } else {
            let render = files_to_show.iter().fold(String::new(), |acc, f| {
                format!("{acc}<a href='{f}'>{f}</a><br>", f = f.display())
            });

            let template = templates::FilesTemplate { content: render };
            format!("HTTP/1.1 200 OK\r\n\r\n{}", template.render().unwrap())
        };

        stream.write_all(response.as_bytes()).await?;

        stream.shutdown().await?;

        Ok(())
    }
}

fn get_files(directory: &PathBuf) -> Vec<PathBuf> {
    read_dir(directory)
        .unwrap()
        .flat_map(|e| {
            let path = e.unwrap().path();
            if path.is_dir() {
                get_files(&path)
            } else {
                vec![path.strip_prefix(directory).unwrap().to_path_buf()]
            }
        })
        .collect()
}
