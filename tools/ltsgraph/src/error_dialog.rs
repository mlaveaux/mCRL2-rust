use slint::slint;

slint!{
    import { StandardButton, Button } from "std-widgets.slint";

    export component ErrorDialog inherits Dialog {
        min-width: 100px;
        min-height: 100px;
        title: error_title;

        in property <string> error_text: "Message";
        in property <string> error_title: "Error!";
    
        VerticalLayout {
            Text {
                vertical-alignment: center;
                text: error_text;
            }    
        }
    
        StandardButton { kind: ok; }
    }
}

/// Shows a standard error dialog with the given title and message, blocking
/// input until closed or "ok" is pressed.
pub fn show_error_dialog(title: &str, message: &str) {
    let dialog = ErrorDialog::new().unwrap();
    dialog.set_error_title(title.into());
    dialog.set_error_text(message.into());

    // Hide the dialog when the Ok button was pressed.
    {
        let weak_dialog = dialog.as_weak();
        dialog.on_ok_clicked(move || {
            if let Some(dialog) = &weak_dialog.upgrade() {
                dialog.hide().unwrap();
            }
        })
    }

    dialog.show().unwrap();
}