/// The drawing tool interface. Each map type provides its own set of brushes.
///
/// A brush is activated when the user enters `Brushing` mode. The editor calls
/// `on_move` every time the cursor moves so the brush can update its live
/// preview, and `on_confirm` when the user presses Enter/Space to commit.
pub trait Brush: Send {
    /// Short display name shown in the brush picker and status bar.
    fn name(&self) -> &str;
    /// Single character used as the preview icon in the brush picker list.
    fn preview_char(&self) -> char;
    /// Called when the cursor moves while the brush is active.
    /// The brush may mutate `app` to update preview cells.
    fn on_move(&mut self, app: &mut crate::app::App);
    /// Called when the user confirms placement (Enter/Space).
    /// Should apply the brush effect and (optionally) trigger a save.
    fn on_confirm(&mut self, app: &mut crate::app::App);
    /// Called when the user cancels (Esc). Clean up any preview state.
    fn cancel(&mut self, app: &mut crate::app::App);
}

/// Placeholder brush used when no real brush is selected yet.
pub struct NullBrush;

impl Brush for NullBrush {
    fn name(&self) -> &str { "(none)" }
    fn preview_char(&self) -> char { ' ' }
    fn on_move(&mut self, _app: &mut crate::app::App) {}
    fn on_confirm(&mut self, app: &mut crate::app::App) {
        app.set_status("No brush selected. Press 'b' to pick one.");
    }
    fn cancel(&mut self, _app: &mut crate::app::App) {}
}
