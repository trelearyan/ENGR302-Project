use std::sync::mpsc::{Receiver, Sender, channel};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum FileOutcome {
    Opened { text: String, name: String },
    Saved { name: String },
    Failed(String),
}

#[derive(Debug)]
pub struct FileChannel {
    sender: Sender<FileOutcome>,
    receiver: Receiver<FileOutcome>,
}

impl Default for FileChannel {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }
}

impl FileChannel {
    pub fn poll(&self) -> Option<FileOutcome> {
        self.receiver.try_recv().ok()
    }

    //load file
    pub fn open(&self, filter_name: &'static str, extensions: &'static [&'static str]) {
        let sender = self.sender.clone();
        let dialog = rfd::AsyncFileDialog::new().add_filter(filter_name, extensions);

        spawn(async move {
            let Some(handle) = dialog.pick_file().await else {
                return;
            };

            let name = handle.file_name();
            let bytes = handle.read().await;

            let outcome = match String::from_utf8(bytes) {
                Ok(text) => FileOutcome::Opened { text, name },
                Err(_) => FileOutcome::Failed(format!("could not read {name}")),
            };

            let _ = sender.send(outcome);
        });
    }

    // save as file
    pub fn save(
        &self,
        text: String,
        suggested_name: String,
        filter_name: &'static str,
        extensions: &'static [&'static str],
    ) {
        save_impl(
            self.sender.clone(),
            text,
            suggested_name,
            filter_name,
            extensions,
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_impl(
    sender: Sender<FileOutcome>,
    text: String,
    suggested_name: String,
    filter_name: &'static str,
    extensions: &'static [&'static str],
) {
    let dialog = rfd::AsyncFileDialog::new()
        .add_filter(filter_name, extensions)
        .set_file_name(suggested_name);
    spawn(async move {
        let Some(handle) = dialog.save_file().await else {
            return;
        };

        let name = handle.file_name();
        let outcome = match handle.write(text.as_bytes()).await {
            Ok(()) => FileOutcome::Saved { name },
            Err(error) => FileOutcome::Failed(format!("could not save {name}: {error}")),
        };

        let _ = sender.send(outcome);
    });
}

#[cfg(target_arch = "wasm32")]
fn save_impl(
    sender: Sender<FileOutcome>,
    text: String,
    suggested_name: String,
    _filter_name: &'static str,
    _extensions: &'static [&'static str],
) {
    let outcome = match download(&text, &suggested_name) {
        Ok(()) => FileOutcome::Saved {
            name: suggested_name,
        },
        Err(reason) => FileOutcome::Failed(reason),
    };
    let _ = sender.send(outcome);
}

fn download(text: &str, filename: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;

    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| "no browser document available".to_owned())?;

    let url = format!("data:text/csv;charset=utf-8,{}", urlencoding::encode(text));

    let anchor = document
        .create_element("a")
        .map_err(|_| "could not prepare the download".to_owned())?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "could not prepare the download".to_owned())?;

    anchor.set_href(&url);
    anchor.set_download(filename);

    if let Some(body) = document.body() {
        let _ = body.append_child(&anchor);
        anchor.click();
        let _ = body.remove_child(&anchor);
    } else {
        anchor.click();
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn<F>(future: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    std::thread::spawn(move || pollster::block_on(future));
}

#[cfg(target_arch = "wasm32")]
fn spawn<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}
