use eframe::egui;
pub fn test(ctx: &egui::Context) {
    ctx.input(|i| {
        for event in &i.events {
            println!("{:?}", event);
        }
    });
}
