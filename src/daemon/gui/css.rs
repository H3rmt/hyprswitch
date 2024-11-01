// language=CSS
pub(super) const CSS: &str = r#"
* {
    border: unset;
}

window {
    --border-color:        rgba(90, 90,110, 0.4);
    --border-color-active: rgba(239, 9,  9, 0.9);
    --bg-color:            rgba(20, 20, 20, 1);
    --bg-color-hover:      rgba(40, 40, 50, 1);
    --index-border-color:  rgba(20,170,170,0.7);
    --border-radius:       12px;
}

.client-image {
    margin: 10px;
}

.client {
    font-size: 16px;
    font-weight: normal;

    border-radius: var(--border-radius);
    background-color: var(--bg-color);
    border: 3px solid var(--border-color);
}

.client:hover {
    background-color: var(--bg-color-hover);
}

.client_active {
    border: 3px solid var(--border-color-active);
}


.workspace {
    font-size: 24px;
    font-weight: bold;

    border-radius: var(--border-radius);
    background-color: var(--bg-color);
    border: 3px solid var(--border-color);
}

.workspace:hover {
    background-color: var(--bg-color-hover);
}

.workspace_special {
    border: 3px solid rgba(0, 255, 0, 0.4);
}

.workspace_active {
    border: 3px solid var(--border-color-active);
}


.monitor {
    opacity: 0.75;
    padding: 2px;

    border-radius: var(--border-radius);
    background-color: var(--bg-color);
    border: 4px solid var(--border-color);
}

.monitor:hover {
    background-color: var(--bg-color-hover);
}

.window_active {
    border: 3px solid var(--border-color-active);
}


.index {
    margin: 3px;
    padding: 2px 4px;
    font-size: 18px;
    font-weight: bold;
    font-family: monospace;
    border-radius: var(--border-radius);
    background-color: var(--bg-color);
    border: 3px solid var(--index-border-color);
}
"#;
