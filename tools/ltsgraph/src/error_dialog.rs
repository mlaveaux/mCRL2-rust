pub fn error_dialog() -> ErrorDialog {
    slint!(
        import { StandardButton, Button } from "std-widgets.slint";

        export component ErrorDialog inherits Dialog {
            min-width: 100px;
            min-height: 100px;
            property <string> error_text: "Error";
        
            VerticalLayout {
                Text {
                vertical-alignment: center;
                text: error_text;
                }    
            }
        
            StandardButton { kind: ok; }
        }
    );
    
    
    ErrorDialog::new()
}