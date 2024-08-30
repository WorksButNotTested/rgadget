use {
    indicatif::{ProgressBar, ProgressFinish, ProgressStyle},
    lazy_static::lazy_static,
    std::sync::Mutex,
};

pub struct Progress {
    verbose: bool,
}

impl Progress {
    pub fn set_verbose(value: bool) {
        let mut progress = Progress::instance().lock().unwrap();
        progress.verbose = value;
    }

    pub fn new(msg: &'static str, length: usize) -> ProgressBar {
        let progress = Progress::instance().lock().unwrap();

        progress.create(msg, length)
    }

    fn instance() -> &'static Mutex<Progress> {
        lazy_static! {
            static ref progress: Mutex<Progress> = Mutex::new(Progress { verbose: false });
        }
        return &progress;
    }

    fn create(&self, msg: &'static str, length: usize) -> ProgressBar {
        if self.verbose {
            let progress_bar = ProgressBar::new(length as u64)
                .with_message(format!("{msg:<50}"))
                .with_finish(ProgressFinish::AndLeave);
            progress_bar.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} [{elapsed_precise:.green}] [{eta_precise:.cyan}] {msg:.magenta} ({percent:.bold}%) [{bar:30.cyan/blue}]",
                    )
                    .unwrap()
                    .progress_chars("█░")
            );
            progress_bar
        } else {
            ProgressBar::hidden()
        }
    }
}
