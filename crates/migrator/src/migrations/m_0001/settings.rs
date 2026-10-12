use serde_json::Value;

pub fn migrate_show_tab_bar_buttons(settings: &mut Value) {
    if let Some(settings) = settings.as_object_mut()
        && let Some(tab_bar) = settings.get_mut("tab_bar").and_then(|v| v.as_object_mut())
        && let Some(show) = tab_bar.get("show_tab_bar_buttons").and_then(|v| v.as_bool())
    {
        let _ = tab_bar.insert(
            "show_tab_bar_buttons".to_string(),
            if show {
                serde_json::Value::String("on_pane_focus".to_string())
            } else {
                serde_json::Value::String("never".to_string())
            },
        );
    }
}

pub fn rename_status_bar_show(settings: &mut Value) {
    if let Some(settings) = settings.as_object_mut()
        && let Some(status_bar) = settings.get_mut("status_bar").and_then(|v| v.as_object_mut())
        && let Some(show) = status_bar.remove("experimental.show")
    {
        status_bar.insert("show".to_string(), show);
    }
}

pub fn removed_settings(settings: &mut Value) {
    if let Some(settings) = settings.as_object_mut() {
        settings.remove("message_editor");
        settings.remove("notification_panel");
        settings.remove("project_name");
    };
}
