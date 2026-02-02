//! Makepad GUI Application for Clash Chain Patcher
//!
//! Dark theme with high contrast text

use makepad_widgets::*;
use clash_chain_patcher::patcher::{self, PatchOptions, Socks5Proxy};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                show_bg: true,
                window: {
                    title: "Clash Chain Patcher",
                    inner_size: vec2(440, 620)
                }
                // Customize Makepad's caption bar to show correct title
                caption_bar = {
                    caption_label = {
                        label = {
                            text: ""
                        }
                    }
                }
                draw_bg: {
                    color: #1a1a1a
                }

                body = <View> {
                    width: Fill,
                    height: Fill,
                    flow: Down,
                    padding: 10,
                    spacing: 6,
                    show_bg: true,
                    draw_bg: {
                        color: #1a1a1a
                    }

                    // Header with logo
                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        align: {x: 0.5, y: 0.5},
                        margin: {bottom: 6},
                        spacing: 8,

                        logo_image = <Image> {
                            width: 32,
                            height: 32,
                            fit: Stretch
                        }

                        <Label> {
                            text: "Clash Chain Patcher"
                            draw_text: {
                                color: #ffffff,
                                text_style: {font_size: 14.0}
                            }
                        }
                    }

                    // Config file
                    <View> {
                        width: Fill,
                        height: Fit,
                        padding: 8,
                        flow: Right,
                        spacing: 6,
                        align: {y: 0.5},
                        show_bg: true,
                        draw_bg: {color: #333333}

                        <Label> {
                            text: "Config"
                            draw_text: {color: #ffffff, text_style: {font_size: 11.0}}
                        }

                        select_file_btn = <Button> {
                            text: "Select"
                            draw_text: {color: #ffffff}
                        }

                        file_label = <Label> {
                            text: "No file"
                            draw_text: {color: #aaaaaa, text_style: {font_size: 10.0}}
                        }
                    }

                    // SOCKS5 Proxy
                    <View> {
                        width: Fill,
                        height: Fit,
                        padding: 8,
                        flow: Down,
                        spacing: 4,
                        show_bg: true,
                        draw_bg: {color: #333333}

                        <Label> {
                            text: "SOCKS5 Proxy"
                            draw_text: {color: #888888, text_style: {font_size: 9.0}}
                        }

                        // Row 1: Server + Port
                        <View> {
                            width: Fill,
                            height: Fit,
                            flow: Right,
                            spacing: 4,
                            align: {y: 0.5},

                            <Label> {
                                width: 40,
                                text: "Host"
                                draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                            }
                            host_input = <TextInput> {
                                width: Fill,
                                height: 24,
                                empty_text: "hostname"
                                draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                                draw_bg: {color: #555555}
                            }
                            <Label> {
                                width: 30,
                                text: "Port"
                                draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                            }
                            port_input = <TextInput> {
                                width: 50,
                                height: 24,
                                empty_text: "1080"
                                draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                                draw_bg: {color: #555555}
                            }
                        }

                        // Row 2: User + Pass
                        <View> {
                            width: Fill,
                            height: Fit,
                            flow: Right,
                            spacing: 4,
                            align: {y: 0.5},

                            <Label> {
                                width: 40,
                                text: "User"
                                draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                            }
                            username_input = <TextInput> {
                                width: Fill,
                                height: 24,
                                empty_text: "user"
                                draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                                draw_bg: {color: #555555}
                            }
                            <Label> {
                                width: 30,
                                text: "Pass"
                                draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                            }
                            password_input = <TextInput> {
                                width: Fill,
                                height: 24,
                                empty_text: "pass"
                                draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                                draw_bg: {color: #555555}
                            }
                        }

                        // Proxy string row
                        <View> {
                            width: Fill,
                            height: Fit,
                            flow: Right,
                            spacing: 4,
                            align: {y: 0.5},
                            margin: {top: 4},

                            proxy_string_input = <TextInput> {
                                width: Fill,
                                height: 24,
                                empty_text: "user:pass@host:port or ip:port:user:pass"
                                draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                                draw_bg: {color: #555555}
                            }

                            fill_btn = <Button> {
                                text: "Fill"
                                draw_text: {color: #ffffff}
                            }
                        }
                    }

                    // Filter
                    <View> {
                        width: Fill,
                        height: Fit,
                        padding: 8,
                        flow: Right,
                        spacing: 4,
                        align: {y: 0.5},
                        show_bg: true,
                        draw_bg: {color: #333333}

                        <Label> {
                            width: 40,
                            text: "Filter"
                            draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                        }
                        filter_input = <TextInput> {
                            width: Fill,
                            height: 24,
                            empty_text: "empty=all, comma separated"
                            draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                            draw_bg: {color: #555555}
                        }
                    }

                    // Buttons
                    <View> {
                        width: Fill,
                        height: Fit,
                        margin: {top: 4},
                        flow: Right,
                        spacing: 6,
                        align: {x: 0.5, y: 0.0},

                        preview_btn = <Button> {
                            text: "Preview"
                            draw_text: {color: #ffffff}
                        }

                        apply_btn = <Button> {
                            text: "Apply"
                            draw_text: {color: #ffffff}
                        }

                        download_btn = <Button> {
                            text: "Save"
                            draw_text: {color: #ffffff}
                        }
                    }

                    // Output
                    <View> {
                        width: Fill,
                        height: Fill,
                        padding: 8,
                        flow: Down,
                        show_bg: true,
                        draw_bg: {color: #333333}

                        <Label> {
                            text: "Output"
                            margin: {bottom: 4},
                            draw_text: {color: #888888, text_style: {font_size: 9.0}}
                        }

                        log_scroll = <ScrollYView> {
                            width: Fill,
                            height: Fill,
                            padding: 6,
                            show_bg: true,
                            draw_bg: {color: #222222}

                            log_text = <Label> {
                                width: Fill,
                                height: Fit,
                                text: "Ready"
                                draw_text: {
                                    color: #00ff00,
                                    text_style: {font_size: 10.0},
                                    wrap: Word
                                }
                            }
                        }
                    }

                    // Status bar with version
                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        align: {y: 0.5},

                        // Version label (left) - will be set dynamically from Cargo.toml
                        version_label = <Label> {
                            text: ""
                            draw_text: {color: #666666, text_style: {font_size: 9.0}}
                        }

                        // Spacer
                        <View> {
                            width: Fill,
                            height: Fit,
                        }

                        // Status label (right)
                        status_label = <Label> {
                            text: "Ready"
                            draw_text: {color: #00ff00, text_style: {font_size: 10.0}}
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default)]
pub struct AppState {
    config_content: Option<String>,
    config_filename: Option<String>,
    output_content: Option<String>,
    logs: Vec<String>,
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,

    #[rust]
    state: AppState,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(id!(select_file_btn)).clicked(actions) {
            self.select_config_file(cx);
        }
        if self.ui.button(id!(fill_btn)).clicked(actions) {
            self.fill_proxy_fields(cx);
        }
        if self.ui.button(id!(preview_btn)).clicked(actions) {
            self.preview_patch(cx);
        }
        if self.ui.button(id!(apply_btn)).clicked(actions) {
            self.apply_patch(cx);
        }
        if self.ui.button(id!(download_btn)).clicked(actions) {
            self.save_output(cx);
        }
    }
}

/// Version from Cargo.toml, set at compile time
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Logo image data embedded at compile time
const LOGO_PNG_DATA: &[u8] = include_bytes!("../logo/logo_32.png");

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        // Set version label and load logo on startup
        if let Event::Startup = event {
            let version_text = format!("v{}", VERSION);
            self.ui.label(id!(version_label)).set_text(cx, &version_text);

            // Load embedded logo image
            if let Err(e) = self.ui.image(id!(logo_image)).load_png_from_data(cx, LOGO_PNG_DATA) {
                eprintln!("Failed to load logo: {:?}", e);
            }
        }
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

impl App {
    fn select_config_file(&mut self, cx: &mut Cx) {
        use rfd::FileDialog;
        let file = FileDialog::new()
            .add_filter("YAML", &["yaml", "yml"])
            .pick_file();

        if let Some(path) = file {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    let filename = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    self.state.config_content = Some(content);
                    self.state.config_filename = Some(filename.clone());
                    self.ui.label(id!(file_label)).set_text(cx, &filename);
                    self.add_log(cx, &format!("Loaded: {}", filename));
                    self.set_status(cx, "Loaded");
                    self.ui.redraw(cx);
                }
                Err(e) => {
                    self.add_log(cx, &format!("Error: {}", e));
                    self.set_status(cx, "Error");
                    self.ui.redraw(cx);
                }
            }
        }
    }

    fn fill_proxy_fields(&mut self, cx: &mut Cx) {
        let proxy_string = self.ui.text_input(id!(proxy_string_input)).text();
        if proxy_string.is_empty() {
            self.add_log(cx, "Enter proxy string first");
            self.ui.redraw(cx);
            return;
        }

        if let Some(proxy) = patcher::parse_proxy_string(&proxy_string) {
            self.ui.text_input(id!(host_input)).set_text(cx, &proxy.host);
            self.ui.text_input(id!(port_input)).set_text(cx, &proxy.port.to_string());
            if let Some(u) = &proxy.username {
                self.ui.text_input(id!(username_input)).set_text(cx, u);
            }
            if let Some(p) = &proxy.password {
                self.ui.text_input(id!(password_input)).set_text(cx, p);
            }
            self.add_log(cx, "Parsed OK");
            self.ui.redraw(cx);
        } else {
            self.add_log(cx, "Invalid format");
            self.ui.redraw(cx);
        }
    }

    fn get_proxy_from_form(&self) -> Option<Socks5Proxy> {
        let host = self.ui.text_input(id!(host_input)).text();
        let port_str = self.ui.text_input(id!(port_input)).text();
        let username = self.ui.text_input(id!(username_input)).text();
        let password = self.ui.text_input(id!(password_input)).text();

        if host.is_empty() || port_str.is_empty() { return None; }
        let port = port_str.parse::<u16>().ok()?;

        Some(Socks5Proxy::new(
            host, port,
            if username.is_empty() { None } else { Some(username) },
            if password.is_empty() { None } else { Some(password) },
        ))
    }

    fn get_options_from_form(&self) -> PatchOptions {
        let filter_str = self.ui.text_input(id!(filter_input)).text();
        let filter_keywords: Vec<String> = if filter_str.is_empty() {
            vec![]
        } else {
            filter_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
        };
        PatchOptions { filter_keywords }
    }

    fn preview_patch(&mut self, cx: &mut Cx) {
        self.clear_logs(cx);
        let config = match &self.state.config_content {
            Some(c) => c.clone(),
            None => { self.add_log(cx, "Select file first"); self.ui.redraw(cx); return; }
        };
        let proxy = match self.get_proxy_from_form() {
            Some(p) => p,
            None => { self.add_log(cx, "Fill proxy info"); self.ui.redraw(cx); return; }
        };
        let opts = self.get_options_from_form();
        let result = patcher::preview_patch(&config, &proxy, &opts);
        for log in &result.logs { self.add_log(cx, log); }
        if result.success {
            self.add_log(cx, "");
            for name in &result.relay_names {
                self.add_log(cx, &format!("  {}", name));
            }
            self.set_status(cx, "Preview OK");
        } else { self.set_status(cx, "Failed"); }
        self.ui.redraw(cx);
    }

    fn apply_patch(&mut self, cx: &mut Cx) {
        self.clear_logs(cx);
        let config = match &self.state.config_content {
            Some(c) => c.clone(),
            None => { self.add_log(cx, "Select file first"); self.ui.redraw(cx); return; }
        };
        let proxy = match self.get_proxy_from_form() {
            Some(p) => p,
            None => { self.add_log(cx, "Fill proxy info"); self.ui.redraw(cx); return; }
        };
        let opts = self.get_options_from_form();
        let result = patcher::apply_patch(&config, &proxy, &opts);
        for log in &result.logs { self.add_log(cx, log); }
        if result.success {
            self.state.output_content = result.output;
            self.add_log(cx, "");
            self.add_log(cx, "Done! Click Save");
            self.set_status(cx, "Done");
        } else { self.set_status(cx, "Failed"); }
        self.ui.redraw(cx);
    }

    fn save_output(&mut self, cx: &mut Cx) {
        let output = match &self.state.output_content {
            Some(o) => o.clone(),
            None => { self.add_log(cx, "Apply first"); self.ui.redraw(cx); return; }
        };
        use rfd::FileDialog;
        let file = FileDialog::new()
            .add_filter("YAML", &["yaml", "yml"])
            .set_file_name("clash_with_chain.yaml")
            .save_file();
        if let Some(path) = file {
            match std::fs::write(&path, output) {
                Ok(_) => {
                    self.add_log(cx, "Saved!");
                    self.set_status(cx, "Saved");
                }
                Err(e) => {
                    self.add_log(cx, &format!("Error: {}", e));
                    self.set_status(cx, "Error");
                }
            }
            self.ui.redraw(cx);
        }
    }

    fn add_log(&mut self, cx: &mut Cx, msg: &str) {
        self.state.logs.push(msg.to_string());
        self.update_log_display(cx);
    }

    fn clear_logs(&mut self, cx: &mut Cx) {
        self.state.logs.clear();
        self.update_log_display(cx);
    }

    fn update_log_display(&mut self, cx: &mut Cx) {
        let text = if self.state.logs.is_empty() { "Ready".to_string() } else { self.state.logs.join("\n") };
        self.ui.label(id!(log_text)).set_text(cx, &text);
    }

    fn set_status(&mut self, cx: &mut Cx, status: &str) {
        self.ui.label(id!(status_label)).set_text(cx, status);
    }
}
