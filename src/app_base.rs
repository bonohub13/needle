#[derive(Debug, Clone)]
pub struct AppBase {
    name: Box<str>,
}

impl AppBase {
    pub fn new(name: &str) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> Box<str> {
        self.name.clone()
    }
}

impl eframe::App for AppBase {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |_| {});
    }
}
