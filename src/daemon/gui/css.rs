pub(super) const CSS: &str = r#"
.client-image {
    margin: 15px;
}
.client {
    border-radius: 15px;
    border: 3px solid rgba(80, 90, 120, 0.80);
    background-color: rgba(25, 25, 25, 0.90);
}
.client:hover {
    background-color: rgba(40, 40, 50, 1);
}
.client_active {
    border: 3px solid rgba(239, 9, 9, 0.94);
}
.workspace {
    font-size: 25px;
    font-weight: bold;
    border-radius: 15px;
    border: 3px solid rgba(70, 80, 90, 0.80);
    background-color: rgba(20, 20, 25, 0.90);
}
.workspace_special {
    border: 3px solid rgba(0, 255, 0, 0.4);
}
.workspace_active {
    border: 3px solid rgba(239, 9, 9, 0.94);
}
.index {
    margin: 6px;
    padding: 5px;
    font-size: 30px;
    font-weight: bold;
    border-radius: 15px;
    border: 3px solid rgba(80, 90, 120, 0.80);
    background-color: rgba(20, 20, 20, 1);
}
.workspaces {
    margin: 10px;
}
window {
    border-radius: 15px;
    opacity: 0.85;
    border: 6px solid rgba(15, 170, 190, 0.85);
}
"#;
