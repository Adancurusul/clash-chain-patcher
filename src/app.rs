//! Makepad GUI Application for Clash Chain Patcher
//!
//! This file contains:
//! - live_design! macro (UI layout)
//! - App struct definition
//! - AppState struct definition
//! - Core Makepad trait implementations
//!
//! Business logic is split into submodules in src/app/:
//! - file_ops.rs: File & Configuration Management
//! - proxy_ops.rs: Proxy Pool Management
//! - health_ops.rs: Health Check Operations
//! - patch_ops.rs: Patch/Apply Operations
//! - ui_helpers.rs: UI & Logging Helpers

use makepad_widgets::*;
use clash_chain_patcher::state::ProxyState;

// impl App blocks are in src/app_impl/ module (declared in main.rs)

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
                    inner_size: vec2(800, 900)
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
                            draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                        }

                        file_label = <Label> {
                            text: "No file"
                            draw_text: {color: #aaaaaa, text_style: {font_size: 10.0}}
                        }

                        clear_config_btn = <Button> {
                            width: Fit,
                            height: Fit,
                            padding: {left: 6, right: 6, top: 2, bottom: 2},
                            text: "×"
                            draw_text: {color: #ff4444, text_style: {font_size: 12.0}}
                            draw_bg: {
                                fn pixel(self) -> vec4 {
                                    return mix(#333333, #555555, self.hover);
                                }
                            }
                        }

                        watch_toggle_btn = <Button> {
                            width: Fit,
                            height: Fit,
                            padding: {left: 8, right: 8, top: 4, bottom: 4},
                            text: "Watch: OFF"
                            draw_text: {color: #ffffff, text_style: {font_size: 9.0}}
                            draw_bg: {
                                fn pixel(self) -> vec4 {
                                    return mix(#555555, #777777, self.hover);
                                }
                            }
                        }

                        <View> { width: Fill, height: Fit }

                        toggle_history_btn = <Button> {
                            width: Fit,
                            height: Fit,
                            padding: {left: 8, right: 8, top: 4, bottom: 4},
                            text: "▼"
                            draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                            draw_bg: {
                                fn pixel(self) -> vec4 {
                                    return mix(#555555, #777777, self.hover);
                                }
                            }
                        }
                    }

                    // Recent files list (collapsible)
                    file_history_view = <View> {
                        visible: false,
                        width: Fill,
                        height: Fit,
                        padding: {left: 16, right: 8, top: 4, bottom: 4},
                        flow: Down,
                        spacing: 2,
                        show_bg: true,
                        draw_bg: {color: #2a2a2a}

                        <Label> {
                            text: "Recent Files:"
                            draw_text: {color: #888888, text_style: {font_size: 9.0}}
                        }

                        recent_file_1 = <Button> {
                            visible: false,
                            width: Fill,
                            height: Fit,
                            padding: {left: 4, right: 4, top: 2, bottom: 2},
                            text: ""
                            draw_text: {color: #aaccff, text_style: {font_size: 9.0}}
                            draw_bg: {
                                fn pixel(self) -> vec4 {
                                    return mix(#2a2a2a, #444444, self.hover);
                                }
                            }
                        }

                        recent_file_2 = <Button> {
                            visible: false,
                            width: Fill,
                            height: Fit,
                            padding: {left: 4, right: 4, top: 2, bottom: 2},
                            text: ""
                            draw_text: {color: #aaccff, text_style: {font_size: 9.0}}
                            draw_bg: {
                                fn pixel(self) -> vec4 {
                                    return mix(#2a2a2a, #444444, self.hover);
                                }
                            }
                        }

                        recent_file_3 = <Button> {
                            visible: false,
                            width: Fill,
                            height: Fit,
                            padding: {left: 4, right: 4, top: 2, bottom: 2},
                            text: ""
                            draw_text: {color: #aaccff, text_style: {font_size: 9.0}}
                            draw_bg: {
                                fn pixel(self) -> vec4 {
                                    return mix(#2a2a2a, #444444, self.hover);
                                }
                            }
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
                                width: 60,
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

                    // Proxy Pool Management
                    <View> {
                        width: Fill,
                        height: 250,
                        padding: 8,
                        flow: Down,
                        spacing: 6,
                        show_bg: true,
                        draw_bg: {color: #333333}

                        // Header
                        <View> {
                            width: Fill,
                            height: Fit,
                            flow: Right,
                            spacing: 6,
                            align: {y: 0.5},

                            <Label> {
                                text: "Proxy Pool"
                                draw_text: {color: #ffffff, text_style: {font_size: 11.0}}
                            }

                            add_proxy_btn = <Button> {
                                text: "+ Add"
                                draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                            }

                            check_all_proxies_btn = <Button> {
                                text: "Check All"
                                draw_text: {color: #ffffff, text_style: {font_size: 10.0}}
                            }

                            auto_check_btn = <Button> {
                                width: Fit,
                                height: Fit,
                                padding: {left: 8, right: 8, top: 4, bottom: 4},
                                text: "Auto: OFF"
                                draw_text: {color: #ffffff, text_style: {font_size: 9.0}}
                                draw_bg: {
                                    fn pixel(self) -> vec4 {
                                        return mix(#555555, #777777, self.hover);
                                    }
                                }
                            }

                            <Label> {
                                text: "Interval:"
                                draw_text: {color: #888888, text_style: {font_size: 9.0}}
                            }

                            interval_input = <TextInput> {
                                width: 50,
                                height: 24,
                                text: "5"
                                empty_text: "5"
                                draw_text: {color: #ffffff, text_style: {font_size: 9.0}}
                                draw_bg: {color: #555555}
                            }

                            <Label> {
                                text: "min"
                                draw_text: {color: #888888, text_style: {font_size: 9.0}}
                            }

                            clear_all_proxies_btn = <Button> {
                                text: "Clear All"
                                draw_text: {color: #ff4444, text_style: {font_size: 10.0}}
                            }

                            <View> { width: Fill, height: Fit }

                            pool_stats_label = <Label> {
                                text: "0 proxies"
                                draw_text: {color: #888888, text_style: {font_size: 9.0}}
                            }
                        }

                        // Proxy Slots (10 pre-allocated)
                        <ScrollYView> {
                            width: Fill,
                            height: Fill,
                            show_bg: true,
                            draw_bg: {color: #222222}

                            <View> {
                                width: Fill,
                                height: Fit,
                                flow: Down,
                                spacing: 2,
                                padding: 4,

                                // Slot 1
                                proxy_slot_1 = <View> {
                                    width: Fill,
                                    height: Fit,
                                    flow: Right,
                                    spacing: 6,
                                    padding: {left: 4, right: 4, top: 3, bottom: 3},
                                    visible: false,
                                    show_bg: true,
                                    draw_bg: {color: #2a2a2a}

                                    proxy_status_1 = <Label> {
                                        text: "⚫"
                                        width: 20,
                                        draw_text: {text_style: {font_size: 10.0}}
                                    }
                                    load_btn_1 = <Button> {
                                        text: "Proxy-1"
                                        width: 200,
                                        draw_text: {color: #aaccff, text_style: {font_size: 9.0}}
                                    }
                                    proxy_info_1 = <Label> {
                                        text: "127.0.0.1:1080"
                                        width: Fill,
                                        draw_text: {color: #888888, text_style: {font_size: 9.0}}
                                    }
                                    check_btn_1 = <Button> {
                                        text: "Check"
                                        draw_text: {color: #ffffff, text_style: {font_size: 9.0}}
                                    }
                                    delete_btn_1 = <Button> {
                                        text: "×"
                                        draw_text: {color: #ff4444, text_style: {font_size: 12.0}}
                                    }
                                }

                                // Slot 2
                                proxy_slot_2 = <View> {
                                    width: Fill,
                                    height: Fit,
                                    flow: Right,
                                    spacing: 6,
                                    padding: {left: 4, right: 4, top: 3, bottom: 3},
                                    visible: false,
                                    show_bg: true,
                                    draw_bg: {color: #2a2a2a}

                                    proxy_status_2 = <Label> { text: "⚫", width: 20, draw_text: {text_style: {font_size: 10.0}} }
                                    load_btn_2 = <Button> { text: "Proxy-2", width: 200, draw_text: {color: #aaccff, text_style: {font_size: 9.0}} }
                                    proxy_info_2 = <Label> { text: "", width: Fill, draw_text: {color: #888888, text_style: {font_size: 9.0}} }
                                    check_btn_2 = <Button> { text: "Check", draw_text: {color: #ffffff, text_style: {font_size: 9.0}} }
                                    delete_btn_2 = <Button> { text: "×", draw_text: {color: #ff4444, text_style: {font_size: 12.0}} }
                                }

                                // Slot 3
                                proxy_slot_3 = <View> {
                                    width: Fill, height: Fit, flow: Right, spacing: 6,
                                    padding: {left: 4, right: 4, top: 3, bottom: 3},
                                    visible: false, show_bg: true, draw_bg: {color: #2a2a2a}
                                    proxy_status_3 = <Label> { text: "⚫", width: 20, draw_text: {text_style: {font_size: 10.0}} }
                                    load_btn_3 = <Button> { text: "Proxy-3", width: 200, draw_text: {color: #aaccff, text_style: {font_size: 9.0}} }
                                    proxy_info_3 = <Label> { text: "", width: Fill, draw_text: {color: #888888, text_style: {font_size: 9.0}} }
                                    check_btn_3 = <Button> { text: "Check", draw_text: {color: #ffffff, text_style: {font_size: 9.0}} }
                                    delete_btn_3 = <Button> { text: "×", draw_text: {color: #ff4444, text_style: {font_size: 12.0}} }
                                }

                                // Slot 4
                                proxy_slot_4 = <View> {
                                    width: Fill, height: Fit, flow: Right, spacing: 6,
                                    padding: {left: 4, right: 4, top: 3, bottom: 3},
                                    visible: false, show_bg: true, draw_bg: {color: #2a2a2a}
                                    proxy_status_4 = <Label> { text: "⚫", width: 20, draw_text: {text_style: {font_size: 10.0}} }
                                    load_btn_4 = <Button> { text: "Proxy-4", width: 200, draw_text: {color: #aaccff, text_style: {font_size: 9.0}} }
                                    proxy_info_4 = <Label> { text: "", width: Fill, draw_text: {color: #888888, text_style: {font_size: 9.0}} }
                                    check_btn_4 = <Button> { text: "Check", draw_text: {color: #ffffff, text_style: {font_size: 9.0}} }
                                    delete_btn_4 = <Button> { text: "×", draw_text: {color: #ff4444, text_style: {font_size: 12.0}} }
                                }

                                // Slot 5
                                proxy_slot_5 = <View> {
                                    width: Fill, height: Fit, flow: Right, spacing: 6,
                                    padding: {left: 4, right: 4, top: 3, bottom: 3},
                                    visible: false, show_bg: true, draw_bg: {color: #2a2a2a}
                                    proxy_status_5 = <Label> { text: "⚫", width: 20, draw_text: {text_style: {font_size: 10.0}} }
                                    load_btn_5 = <Button> { text: "Proxy-5", width: 200, draw_text: {color: #aaccff, text_style: {font_size: 9.0}} }
                                    proxy_info_5 = <Label> { text: "", width: Fill, draw_text: {color: #888888, text_style: {font_size: 9.0}} }
                                    check_btn_5 = <Button> { text: "Check", draw_text: {color: #ffffff, text_style: {font_size: 9.0}} }
                                    delete_btn_5 = <Button> { text: "×", draw_text: {color: #ff4444, text_style: {font_size: 12.0}} }
                                }

                                // Slot 6
                                proxy_slot_6 = <View> {
                                    width: Fill, height: Fit, flow: Right, spacing: 6,
                                    padding: {left: 4, right: 4, top: 3, bottom: 3},
                                    visible: false, show_bg: true, draw_bg: {color: #2a2a2a}
                                    proxy_status_6 = <Label> { text: "⚫", width: 20, draw_text: {text_style: {font_size: 10.0}} }
                                    load_btn_6 = <Button> { text: "Proxy-6", width: 200, draw_text: {color: #aaccff, text_style: {font_size: 9.0}} }
                                    proxy_info_6 = <Label> { text: "", width: Fill, draw_text: {color: #888888, text_style: {font_size: 9.0}} }
                                    check_btn_6 = <Button> { text: "Check", draw_text: {color: #ffffff, text_style: {font_size: 9.0}} }
                                    delete_btn_6 = <Button> { text: "×", draw_text: {color: #ff4444, text_style: {font_size: 12.0}} }
                                }

                                // Slot 7
                                proxy_slot_7 = <View> {
                                    width: Fill, height: Fit, flow: Right, spacing: 6,
                                    padding: {left: 4, right: 4, top: 3, bottom: 3},
                                    visible: false, show_bg: true, draw_bg: {color: #2a2a2a}
                                    proxy_status_7 = <Label> { text: "⚫", width: 20, draw_text: {text_style: {font_size: 10.0}} }
                                    load_btn_7 = <Button> { text: "Proxy-7", width: 200, draw_text: {color: #aaccff, text_style: {font_size: 9.0}} }
                                    proxy_info_7 = <Label> { text: "", width: Fill, draw_text: {color: #888888, text_style: {font_size: 9.0}} }
                                    check_btn_7 = <Button> { text: "Check", draw_text: {color: #ffffff, text_style: {font_size: 9.0}} }
                                    delete_btn_7 = <Button> { text: "×", draw_text: {color: #ff4444, text_style: {font_size: 12.0}} }
                                }

                                // Slot 8
                                proxy_slot_8 = <View> {
                                    width: Fill, height: Fit, flow: Right, spacing: 6,
                                    padding: {left: 4, right: 4, top: 3, bottom: 3},
                                    visible: false, show_bg: true, draw_bg: {color: #2a2a2a}
                                    proxy_status_8 = <Label> { text: "⚫", width: 20, draw_text: {text_style: {font_size: 10.0}} }
                                    load_btn_8 = <Button> { text: "Proxy-8", width: 200, draw_text: {color: #aaccff, text_style: {font_size: 9.0}} }
                                    proxy_info_8 = <Label> { text: "", width: Fill, draw_text: {color: #888888, text_style: {font_size: 9.0}} }
                                    check_btn_8 = <Button> { text: "Check", draw_text: {color: #ffffff, text_style: {font_size: 9.0}} }
                                    delete_btn_8 = <Button> { text: "×", draw_text: {color: #ff4444, text_style: {font_size: 12.0}} }
                                }

                                // Slot 9
                                proxy_slot_9 = <View> {
                                    width: Fill, height: Fit, flow: Right, spacing: 6,
                                    padding: {left: 4, right: 4, top: 3, bottom: 3},
                                    visible: false, show_bg: true, draw_bg: {color: #2a2a2a}
                                    proxy_status_9 = <Label> { text: "⚫", width: 20, draw_text: {text_style: {font_size: 10.0}} }
                                    load_btn_9 = <Button> { text: "Proxy-9", width: 200, draw_text: {color: #aaccff, text_style: {font_size: 9.0}} }
                                    proxy_info_9 = <Label> { text: "", width: Fill, draw_text: {color: #888888, text_style: {font_size: 9.0}} }
                                    check_btn_9 = <Button> { text: "Check", draw_text: {color: #ffffff, text_style: {font_size: 9.0}} }
                                    delete_btn_9 = <Button> { text: "×", draw_text: {color: #ff4444, text_style: {font_size: 12.0}} }
                                }

                                // Slot 10
                                proxy_slot_10 = <View> {
                                    width: Fill, height: Fit, flow: Right, spacing: 6,
                                    padding: {left: 4, right: 4, top: 3, bottom: 3},
                                    visible: false, show_bg: true, draw_bg: {color: #2a2a2a}
                                    proxy_status_10 = <Label> { text: "⚫", width: 20, draw_text: {text_style: {font_size: 10.0}} }
                                    load_btn_10 = <Button> { text: "Proxy-10", width: 200, draw_text: {color: #aaccff, text_style: {font_size: 9.0}} }
                                    proxy_info_10 = <Label> { text: "", width: Fill, draw_text: {color: #888888, text_style: {font_size: 9.0}} }
                                    check_btn_10 = <Button> { text: "Check", draw_text: {color: #ffffff, text_style: {font_size: 9.0}} }
                                    delete_btn_10 = <Button> { text: "×", draw_text: {color: #ff4444, text_style: {font_size: 12.0}} }
                                }

                                // Empty message (shown when no proxies)
                                proxy_empty_msg = <View> {
                                    width: Fill,
                                    height: Fill,
                                    align: {x: 0.5, y: 0.5},
                                    visible: true,

                                    <Label> {
                                        text: "No proxies in pool"
                                        draw_text: {color: #666666, text_style: {font_size: 10.0}}
                                    }
                                }
                            }
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

/// Result of an Apply operation
#[derive(Debug)]
pub struct ApplyResult {
    pub success: bool,
    pub message: String,
    pub details: Vec<String>,
}

/// Application state
pub struct AppState {
    pub config_content: Option<String>,
    pub config_filename: Option<String>,
    pub output_content: Option<String>,
    pub logs: Vec<String>,
    pub proxy_state: Option<ProxyState>,
    #[allow(dead_code)]
    pub checking: bool,
    #[allow(dead_code)]
    pub selected_proxy_index: Option<usize>,
    pub watching: bool,
    pub recent_files: Vec<String>,
    pub show_file_history: bool,
    pub auto_checking: bool,
    pub auto_check_interval: u64,
    pub health_check_rx: Option<std::sync::mpsc::Receiver<(String, clash_chain_patcher::health::ProxyValidationResult)>>,
    #[allow(dead_code)]
    pub auto_check_handle: Option<tokio::task::JoinHandle<()>>,
    pub watcher_rx: Option<tokio::sync::mpsc::UnboundedReceiver<clash_chain_patcher::watcher::WatcherEvent>>,
    pub watcher_bridge: Option<clash_chain_patcher::bridge::WatcherBridge>,
    pub apply_result_rx: Option<std::sync::mpsc::Receiver<ApplyResult>>,
    pub is_applying: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config_content: None,
            config_filename: None,
            output_content: None,
            logs: Vec::new(),
            proxy_state: None,
            checking: false,
            selected_proxy_index: None,
            watching: false,
            recent_files: Vec::new(),
            show_file_history: false,
            auto_checking: false,
            auto_check_interval: 5,
            health_check_rx: None,
            auto_check_handle: None,
            watcher_rx: None,
            watcher_bridge: None,
            apply_result_rx: None,
            is_applying: false,
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    pub ui: WidgetRef,

    #[rust]
    pub state: AppState,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
    }
}

/// Version from Cargo.toml, set at compile time
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Logo image data embedded at compile time
const LOGO_PNG_DATA: &[u8] = include_bytes!("../logo/logo_32.png");

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(id!(select_file_btn)).clicked(actions) {
            self.select_config_file(cx);
        }
        if self.ui.button(id!(clear_config_btn)).clicked(actions) {
            self.clear_config_file(cx);
        }
        if self.ui.button(id!(watch_toggle_btn)).clicked(actions) {
            self.toggle_watch(cx);
        }
        if self.ui.button(id!(toggle_history_btn)).clicked(actions) {
            self.toggle_file_history(cx);
        }
        // Recent file buttons
        if self.ui.button(id!(recent_file_1)).clicked(actions) {
            self.select_recent_file(cx, 0);
        }
        if self.ui.button(id!(recent_file_2)).clicked(actions) {
            self.select_recent_file(cx, 1);
        }
        if self.ui.button(id!(recent_file_3)).clicked(actions) {
            self.select_recent_file(cx, 2);
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
        if self.ui.button(id!(add_proxy_btn)).clicked(actions) {
            self.add_proxy_to_pool(cx);
        }
        if self.ui.button(id!(check_all_proxies_btn)).clicked(actions) {
            self.check_all_proxies(cx);
        }
        if self.ui.button(id!(auto_check_btn)).clicked(actions) {
            self.toggle_auto_health_check(cx);
        }
        if self.ui.button(id!(clear_all_proxies_btn)).clicked(actions) {
            self.clear_all_proxies(cx);
        }

        // Individual proxy slot buttons
        for slot in 1..=10 {
            let load_btn_id = match slot {
                1 => id!(load_btn_1), 2 => id!(load_btn_2), 3 => id!(load_btn_3),
                4 => id!(load_btn_4), 5 => id!(load_btn_5), 6 => id!(load_btn_6),
                7 => id!(load_btn_7), 8 => id!(load_btn_8), 9 => id!(load_btn_9),
                10 => id!(load_btn_10), _ => continue,
            };
            let check_btn_id = match slot {
                1 => id!(check_btn_1), 2 => id!(check_btn_2), 3 => id!(check_btn_3),
                4 => id!(check_btn_4), 5 => id!(check_btn_5), 6 => id!(check_btn_6),
                7 => id!(check_btn_7), 8 => id!(check_btn_8), 9 => id!(check_btn_9),
                10 => id!(check_btn_10), _ => continue,
            };
            let delete_btn_id = match slot {
                1 => id!(delete_btn_1), 2 => id!(delete_btn_2), 3 => id!(delete_btn_3),
                4 => id!(delete_btn_4), 5 => id!(delete_btn_5), 6 => id!(delete_btn_6),
                7 => id!(delete_btn_7), 8 => id!(delete_btn_8), 9 => id!(delete_btn_9),
                10 => id!(delete_btn_10), _ => continue,
            };

            if self.ui.button(load_btn_id).clicked(actions) {
                self.load_proxy_to_form(cx, slot - 1);
            }
            if self.ui.button(check_btn_id).clicked(actions) {
                self.check_proxy_by_slot(cx, slot - 1);
            }
            if self.ui.button(delete_btn_id).clicked(actions) {
                self.delete_proxy_by_slot(cx, slot - 1);
            }
        }
    }
}

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

            // Initialize ProxyState
            self.init_proxy_state(cx);

            // Set default interval
            self.ui.text_input(id!(interval_input)).set_text(cx, "5");
            self.state.auto_check_interval = 5;
        }

        // Check for background health check results
        let mut results = Vec::new();
        if let Some(rx) = &self.state.health_check_rx {
            while let Ok(result) = rx.try_recv() {
                results.push(result);
            }
        }

        // Process results (after releasing borrow)
        for (proxy_id, result) in results {
            self.update_proxy_health_from_background(cx, proxy_id, result);
        }

        // Check for file watcher events
        let mut watcher_events = Vec::new();
        if let Some(rx) = &mut self.state.watcher_rx {
            while let Ok(event) = rx.try_recv() {
                watcher_events.push(event);
            }
        }

        // Process file watcher events
        for event in watcher_events {
            self.handle_watcher_event(cx, event);
        }

        // Check for Apply operation results
        if let Some(rx) = &self.state.apply_result_rx {
            if let Ok(result) = rx.try_recv() {
                self.handle_apply_result(cx, result);
                self.state.apply_result_rx = None;
                self.state.is_applying = false;
            }
        }

        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
