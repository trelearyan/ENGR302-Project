use std::sync::mpsc::{channel, Receiver, Sender};

pub enum FileOutcome {
    Opened { text: String, name: String },
    Saved { name: String },
    Failed(String),
}

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
        let sender = self.sender.clone();
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