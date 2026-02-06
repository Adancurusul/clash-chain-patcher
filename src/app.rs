//! Makepad GUI Application for Clash Chain Patcher
//!
//! Dark theme with high contrast text

use makepad_widgets::*;
use clash_chain_patcher::patcher::{self, PatchOptions, Socks5Proxy};
use clash_chain_patcher::state::ProxyState;
use clash_chain_patcher::config::UpstreamProxy;
use clash_chain_patcher::proxy::config::UpstreamConfig;

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

pub struct AppState {
    config_content: Option<String>,
    config_filename: Option<String>,
    output_content: Option<String>,
    logs: Vec<String>,
    proxy_state: Option<ProxyState>,
    #[allow(dead_code)] // Will be used for edit/delete operations
    checking: bool,
    #[allow(dead_code)] // Reserved for future proxy selection feature
    selected_proxy_index: Option<usize>,
    watching: bool,
    /// Recent Clash config file paths (max 5)
    recent_files: Vec<String>,
    /// Whether file history dropdown is shown
    show_file_history: bool,
    /// Whether auto health check is running
    auto_checking: bool,
    /// Auto check interval in minutes
    auto_check_interval: u64,
    /// Health check results channel (for background updates)
    health_check_rx: Option<std::sync::mpsc::Receiver<(String, clash_chain_patcher::health::ProxyValidationResult)>>,
    /// Auto check task handle
    #[allow(dead_code)]
    auto_check_handle: Option<tokio::task::JoinHandle<()>>,
    /// File watcher event channel receiver
    watcher_rx: Option<tokio::sync::mpsc::UnboundedReceiver<clash_chain_patcher::watcher::WatcherEvent>>,
    /// File watcher bridge instance
    watcher_bridge: Option<clash_chain_patcher::bridge::WatcherBridge>,
    /// Apply operation result channel
    apply_result_rx: Option<std::sync::mpsc::Receiver<ApplyResult>>,
    /// Whether an Apply operation is in progress
    is_applying: bool,
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
            auto_check_interval: 5, // Default 5 minutes
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

impl App {
    fn select_config_file(&mut self, cx: &mut Cx) {
        use rfd::FileDialog;
        let file = FileDialog::new()
            .add_filter("YAML", &["yaml", "yml"])
            .pick_file();

        if let Some(path) = file {
            let path_str = path.to_string_lossy().to_string();
            self.load_config_file(cx, path_str);
        }
    }

    fn load_config_file(&mut self, cx: &mut Cx, path_str: String) {
        let path = std::path::Path::new(&path_str);

        match std::fs::read_to_string(path) {
            Ok(content) => {
                let filename = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                self.state.config_content = Some(content);
                self.state.config_filename = Some(filename.clone());

                // Add to recent files (if not already there)
                self.add_to_recent_files(cx, path_str.clone());

                // Set Clash config path for proxy pool merging
                if let Some(state) = &mut self.state.proxy_state {
                    state.set_clash_config_path(path.to_path_buf());
                    self.add_log(cx, &format!("✓ Loaded: {}", filename));
                    self.add_log(cx, "  Clash config path set for proxy pool");
                } else {
                    self.add_log(cx, &format!("Loaded: {}", filename));
                }

                self.ui.label(id!(file_label)).set_text(cx, &filename);
                self.set_status(cx, "Loaded");
                self.refresh_file_history_display(cx);
                self.ui.redraw(cx);
            }
            Err(e) => {
                self.add_log(cx, &format!("Error: {}", e));
                self.set_status(cx, "Error");
                self.ui.redraw(cx);
            }
        }
    }

    fn clear_config_file(&mut self, cx: &mut Cx) {
        self.state.config_content = None;
        self.state.config_filename = None;

        // Clear Clash config path from ProxyState
        if let Some(_state) = &mut self.state.proxy_state {
            // Note: ProxyState doesn't have a clear method, but we can just not use it
            self.clear_logs(cx);
            self.add_log(cx, "✓ Cleared Clash config selection");
        }

        self.ui.label(id!(file_label)).set_text(cx, "No file");
        self.set_status(cx, "Ready");
        self.ui.redraw(cx);
    }

    fn toggle_watch(&mut self, cx: &mut Cx) {
        eprintln!("DEBUG: toggle_watch called, current watching = {}", self.state.watching);
        self.state.watching = !self.state.watching;

        let button_text = if self.state.watching {
            "Watch: ON"
        } else {
            "Watch: OFF"
        };

        eprintln!("DEBUG: Setting button text to: {}", button_text);
        self.ui.button(id!(watch_toggle_btn)).set_text(cx, button_text);

        self.clear_logs(cx);
        if self.state.watching {
            // Start file watcher - extract config_path first to avoid borrow conflict
            let config_path_opt = self.state.proxy_state
                .as_ref()
                .and_then(|state| state.clash_config_path())
                .map(|p| p.to_path_buf());

            if let Some(config_path) = config_path_opt {
                use clash_chain_patcher::bridge::WatcherBridge;

                match WatcherBridge::new(&config_path) {
                    Ok(mut bridge) => {
                        match bridge.start() {
                            Ok(rx) => {
                                self.state.watcher_rx = Some(rx);
                                self.state.watcher_bridge = Some(bridge);
                                self.add_log(cx, "✓ File watching enabled");
                                self.add_log(cx, &format!("  Monitoring: {}", config_path.display()));
                                self.add_log(cx, "  Will auto re-apply on external changes");
                                eprintln!("DEBUG: File watcher started successfully");
                            }
                            Err(e) => {
                                self.state.watching = false;
                                self.ui.button(id!(watch_toggle_btn)).set_text(cx, "Watch: OFF");
                                self.add_log(cx, &format!("✗ Failed to start watcher: {}", e));
                                eprintln!("ERROR: Failed to start watcher: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.state.watching = false;
                        self.ui.button(id!(watch_toggle_btn)).set_text(cx, "Watch: OFF");
                        self.add_log(cx, &format!("✗ Failed to create watcher: {}", e));
                        eprintln!("ERROR: Failed to create watcher: {}", e);
                    }
                }
            } else {
                self.state.watching = false;
                self.ui.button(id!(watch_toggle_btn)).set_text(cx, "Watch: OFF");
                self.add_log(cx, "✗ No Clash config file selected");
                self.add_log(cx, "  Select a file first, then enable Watch");
            }
        } else {
            // Stop file watcher
            if let Some(mut bridge) = self.state.watcher_bridge.take() {
                bridge.stop();
                self.state.watcher_rx = None;
                self.add_log(cx, "File watching disabled");
                eprintln!("DEBUG: File watcher stopped");
            }
        }

        self.ui.redraw(cx);
    }

    fn toggle_file_history(&mut self, cx: &mut Cx) {
        eprintln!("DEBUG: toggle_file_history called, current state = {}", self.state.show_file_history);
        self.state.show_file_history = !self.state.show_file_history;

        // Toggle visibility
        self.ui.view(id!(file_history_view)).set_visible(cx, self.state.show_file_history);

        // Update button text
        let button_text = if self.state.show_file_history { "▲" } else { "▼" };
        self.ui.button(id!(toggle_history_btn)).set_text(cx, button_text);

        // Show feedback in logs
        if self.state.show_file_history {
            eprintln!("DEBUG: Showing file history, {} recent files", self.state.recent_files.len());
            if self.state.recent_files.is_empty() {
                self.clear_logs(cx);
                self.add_log(cx, "No recent files yet");
                self.add_log(cx, "Select a Clash config file to add it to history");
            }
        } else {
            eprintln!("DEBUG: Hiding file history");
        }

        self.ui.redraw(cx);
    }

    fn add_to_recent_files(&mut self, cx: &mut Cx, path: String) {
        // Save to persistent config through ProxyState
        if let Some(state) = &mut self.state.proxy_state {
            if let Err(e) = state.add_recent_file(path.clone()) {
                eprintln!("Failed to save recent file: {}", e);
            } else {
                // Update in-memory list from saved config
                self.state.recent_files = state.get_recent_files();
                eprintln!("DEBUG: Saved recent file, now have {} files", self.state.recent_files.len());
            }
        }

        // Refresh display
        self.refresh_file_history_display(cx);
    }

    fn refresh_file_history_display(&mut self, cx: &mut Cx) {
        // Update recent file buttons
        for i in 0..3 {
            let (btn_id, visible, text) = match i {
                0 => (id!(recent_file_1), self.state.recent_files.len() > 0, self.state.recent_files.get(0)),
                1 => (id!(recent_file_2), self.state.recent_files.len() > 1, self.state.recent_files.get(1)),
                2 => (id!(recent_file_3), self.state.recent_files.len() > 2, self.state.recent_files.get(2)),
                _ => continue,
            };

            if visible {
                if let Some(path) = text {
                    // Show filename only (not full path)
                    let filename = std::path::Path::new(path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.clone());

                    self.ui.button(btn_id).set_text(cx, &filename);
                    self.ui.button(btn_id).set_visible(cx, true);
                }
            } else {
                self.ui.button(btn_id).set_visible(cx, false);
            }
        }
    }

    fn select_recent_file(&mut self, cx: &mut Cx, index: usize) {
        if let Some(path) = self.state.recent_files.get(index).cloned() {
            self.load_config_file(cx, path);
            // Hide history after selection
            self.state.show_file_history = false;
            self.ui.view(id!(file_history_view)).set_visible(cx, false);
            self.ui.button(id!(toggle_history_btn)).set_text(cx, "▼");
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
        // Check if already applying
        if self.state.is_applying {
            self.add_log(cx, "⚠ Apply is already in progress");
            return;
        }

        // Check if config file is selected
        if self.state.config_filename.is_none() {
            self.add_log(cx, "✗ Select file first");
            return;
        }

        // Check ProxyState
        let Some(state) = &self.state.proxy_state else {
            self.add_log(cx, "✗ ProxyState not initialized");
            return;
        };

        // Get enabled proxies
        let enabled_proxies: Vec<_> = state.list_upstreams()
            .into_iter()
            .filter(|p| p.enabled)
            .collect();

        if enabled_proxies.is_empty() {
            self.add_log(cx, "✗ No enabled proxies");
            self.add_log(cx, "  Add and enable at least 1 proxy");
            return;
        }

        // Extract config path
        let config_path = state.clash_config_path()
            .map(|p| p.to_path_buf());

        let Some(config_path) = config_path else {
            self.add_log(cx, "✗ Clash config path not set");
            return;
        };

        // Create channel for result
        let (tx, rx) = std::sync::mpsc::channel();
        self.state.apply_result_rx = Some(rx);
        self.state.is_applying = true;

        // Display progress
        self.clear_logs(cx);
        self.add_log(cx, "⏳ Applying configuration...");
        self.add_log(cx, &format!("  {} enabled proxies", enabled_proxies.len()));
        self.add_log(cx, "  (Non-blocking, UI remains responsive)");
        self.set_status(cx, "Applying...");

        // Spawn background thread
        std::thread::spawn(move || {
            use clash_chain_patcher::bridge::MergerBridge;

            let result = (|| -> Result<ApplyResult, String> {
                // Create MergerBridge
                let merger = MergerBridge::new();

                // Execute merge
                match merger.merge(&config_path) {
                    Ok(merge_result) => {
                        let mut details = Vec::new();
                        details.push("Using proxy pool mode".to_string());
                        details.push(format!("Enabled proxies: {}", enabled_proxies.len()));

                        for proxy in &enabled_proxies {
                            details.push(format!("  - {}", proxy.name));
                        }

                        details.push("".to_string());
                        details.push(format!("Proxy added: {}", merge_result.proxy_added));
                        details.push(format!("Groups updated: {}", merge_result.groups_updated));

                        if let Some(backup_path) = merge_result.backup_path {
                            details.push(format!("Backup: {}", backup_path.display()));
                        }

                        Ok(ApplyResult {
                            success: true,
                            message: "✓ Configuration applied successfully".to_string(),
                            details,
                        })
                    }
                    Err(e) => {
                        Ok(ApplyResult {
                            success: false,
                            message: format!("✗ Apply failed: {}", e),
                            details: vec![],
                        })
                    }
                }
            })();

            let apply_result = result.unwrap_or_else(|e| ApplyResult {
                success: false,
                message: format!("✗ Error: {}", e),
                details: vec![],
            });

            let _ = tx.send(apply_result);
        });

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

    fn handle_watcher_event(&mut self, cx: &mut Cx, event: clash_chain_patcher::watcher::WatcherEvent) {
        use clash_chain_patcher::watcher::WatcherEvent;

        match event {
            WatcherEvent::ConfigModified(path) | WatcherEvent::ConfigCreated(path) => {
                eprintln!("DEBUG: Config file modified: {}", path.display());
                self.add_log(cx, "⚠ Clash config file was modified externally");
                self.add_log(cx, &format!("  File: {}", path.display()));
                self.add_log(cx, "  Re-applying Local-Chain-Proxy...");

                // Re-apply the configuration
                if let Some(state) = &mut self.state.proxy_state {
                    match state.merge_to_clash() {
                        Ok(_) => {
                            self.add_log(cx, "✓ Auto re-applied successfully");
                            self.add_log(cx, "  Local-Chain-Proxy restored");
                        }
                        Err(e) => {
                            self.add_log(cx, &format!("✗ Auto re-apply failed: {}", e));
                        }
                    }
                } else {
                    self.add_log(cx, "✗ ProxyState not initialized");
                }

                self.ui.redraw(cx);
            }
            WatcherEvent::Error(error) => {
                eprintln!("ERROR: File watcher error: {}", error);
                self.add_log(cx, &format!("✗ Watcher error: {}", error));
                self.ui.redraw(cx);
            }
        }
    }

    fn handle_apply_result(&mut self, cx: &mut Cx, result: ApplyResult) {
        eprintln!("DEBUG: Apply completed: success={}", result.success);

        self.clear_logs(cx);
        self.add_log(cx, &result.message);

        for detail in result.details {
            self.add_log(cx, &detail);
        }

        if result.success {
            self.add_log(cx, "");
            self.add_log(cx, "Local proxy: 127.0.0.1:10808");
            self.add_log(cx, "");
            self.add_log(cx, "Next steps:");
            self.add_log(cx, "1. Refresh Clash configuration");
            self.add_log(cx, "2. Select 'Local-Chain-Proxy' in Clash");
            self.add_log(cx, "3. Enable Watch to protect against subscription updates");
            self.set_status(cx, "Done");
        } else {
            self.set_status(cx, "Failed");
        }

        self.ui.redraw(cx);
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

    // Proxy Pool Management Methods

    fn init_proxy_state(&mut self, cx: &mut Cx) {
        let mut state = ProxyState::new();
        if let Err(e) = state.initialize() {
            self.add_log(cx, &format!("ProxyState init error: {}", e));
            return;
        }

        // Debug: Check how many proxies were loaded
        let proxy_count = state.list_upstreams().len();
        eprintln!("DEBUG: Loaded {} proxies from config", proxy_count);

        self.state.proxy_state = Some(state);

        // Load recent files from config
        if let Some(state) = &self.state.proxy_state {
            self.state.recent_files = state.get_recent_files();
            eprintln!("DEBUG: Loaded {} recent files from config", self.state.recent_files.len());
            self.refresh_file_history_display(cx);
        }

        // Refresh display will show proxy list (and clear logs to show current state)
        self.refresh_proxy_list_display(cx);

        // Add initialization message if no proxies exist
        if let Some(state) = &self.state.proxy_state {
            let proxies = state.list_upstreams();
            eprintln!("DEBUG: After setting state, proxy count = {}", proxies.len());
            if proxies.is_empty() {
                self.add_log(cx, "✓ ProxyState initialized - No proxies configured yet");
            }
        }
    }

    fn add_proxy_to_pool(&mut self, cx: &mut Cx) {
        // Get proxy from form
        let proxy = match self.get_proxy_from_form() {
            Some(p) => p,
            None => {
                self.clear_logs(cx);
                self.add_log(cx, "✗ Please fill proxy info first");
                self.ui.redraw(cx);
                return;
            }
        };

        // Check for duplicates (same host:port)
        if let Some(state) = &self.state.proxy_state {
            let exists = state.list_upstreams()
                .iter()
                .any(|p| p.config.host == proxy.host && p.config.port == proxy.port);

            if exists {
                self.clear_logs(cx);
                self.add_log(cx, &format!("✗ Proxy {}:{} already exists!", proxy.host, proxy.port));
                self.refresh_proxy_list_display(cx);
                self.ui.redraw(cx);
                return;
            }
        }

        // Get a name - use host:port format
        let name = format!("{}:{}", proxy.host, proxy.port);

        // Convert to UpstreamProxy
        let upstream = UpstreamProxy {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.clone(),
            config: UpstreamConfig {
                host: proxy.host.clone(),
                port: proxy.port,
                username: proxy.username.clone(),
                password: proxy.password.clone(),
            },
            enabled: true,
            health: Default::default(),
        };

        // Add to pool
        if let Some(state) = &mut self.state.proxy_state {
            match state.add_upstream(upstream) {
                Ok(_) => {
                    self.clear_logs(cx);
                    self.add_log(cx, &format!("✓ Added proxy: {}", name));
                    self.refresh_proxy_list_display(cx);
                }
                Err(e) => {
                    self.clear_logs(cx);
                    self.add_log(cx, &format!("✗ Add error: {}", e));
                }
            }
        } else {
            self.clear_logs(cx);
            self.add_log(cx, "✗ ProxyState not initialized!");
        }
        self.ui.redraw(cx);
    }

    fn check_all_proxies(&mut self, cx: &mut Cx) {
        if self.state.checking {
            self.add_log(cx, "Check in progress...");
            self.ui.redraw(cx);
            return;
        }

        // Get proxies to check
        let proxies_info: Vec<(String, String, u16, Option<String>, Option<String>)> = {
            if let Some(state) = &self.state.proxy_state {
                state.list_upstreams()
                    .into_iter()
                    .filter(|p| p.enabled)
                    .map(|p| (
                        p.id.clone(),
                        p.config.host.clone(),
                        p.config.port,
                        p.config.username.clone(),
                        p.config.password.clone(),
                    ))
                    .collect()
            } else {
                Vec::new()
            }
        };

        if proxies_info.is_empty() {
            self.clear_logs(cx);
            self.add_log(cx, "No enabled proxies to check");
            self.ui.redraw(cx);
            return;
        }

        self.state.checking = true;
        self.clear_logs(cx);
        self.add_log(cx, &format!("Checking {} proxies...", proxies_info.len()));
        self.add_log(cx, "Note: UI may freeze briefly (10s per proxy)");
        self.ui.redraw(cx);

        // Use enhanced validator (same as individual check)
        use clash_chain_patcher::health::ProxyValidator;
        let validator = ProxyValidator::new(10);

        for (i, (proxy_id, host, port, username, password)) in proxies_info.iter().enumerate() {
            self.add_log(cx, &format!("Checking {}/{}...", i + 1, proxies_info.len()));
            self.ui.redraw(cx);

            let result = validator.validate(
                host,
                *port,
                username.as_deref(),
                password.as_deref(),
            );

            // Update proxy health
            if let Some(state) = &mut self.state.proxy_state {
                if let Some(mut proxy) = state.get_upstream(proxy_id) {
                    if result.is_valid {
                        if let Some(latency) = result.latency_ms {
                            proxy.health.mark_healthy_with_details(
                                latency as u64,
                                result.exit_ip,
                                result.location.as_ref().map(|l| l.format_short()),
                                result.location.as_ref().map(|l| l.country_code.clone()),
                            );
                        }
                    } else if let Some(error) = result.error {
                        proxy.health.mark_unhealthy(error);
                    }
                    let _ = state.update_upstream(proxy);
                }
            }
        }

        self.state.checking = false;
        self.clear_logs(cx);
        self.add_log(cx, "✓ Health check completed");
        self.refresh_proxy_list_display(cx);
        self.ui.redraw(cx);
    }

    fn toggle_auto_health_check(&mut self, cx: &mut Cx) {
        eprintln!("DEBUG: toggle_auto_health_check called, current state = {}", self.state.auto_checking);

        // Read interval from input
        let interval_str = self.ui.text_input(id!(interval_input)).text();
        let interval_minutes = interval_str.parse::<u64>().unwrap_or(5);
        self.state.auto_check_interval = interval_minutes;

        if self.state.auto_checking {
            // Stop auto checking
            self.state.auto_checking = false;
            self.state.health_check_rx = None;
            // Task will be dropped and cancelled

            self.ui.button(id!(auto_check_btn)).set_text(cx, "Auto: OFF");
            self.clear_logs(cx);
            self.add_log(cx, "Auto health check stopped");

            eprintln!("DEBUG: Auto check stopped");
        } else {
            // Start auto checking
            use std::sync::mpsc;

            let (tx, rx) = mpsc::channel();
            self.state.health_check_rx = Some(rx);

            // Get proxies info for background check
            if let Some(state) = &self.state.proxy_state {
                let proxies = state.list_upstreams();

                if proxies.is_empty() {
                    self.clear_logs(cx);
                    self.add_log(cx, "✗ No proxies to check");
                    self.add_log(cx, "  Add proxies first");
                    return;
                }

                // Create shared proxy list
                let proxy_list: Vec<_> = proxies.into_iter()
                    .filter(|p| p.enabled)
                    .map(|p| (
                        p.id.clone(),
                        p.config.host.clone(),
                        p.config.port,
                        p.config.username.clone(),
                        p.config.password.clone(),
                    ))
                    .collect();

                if proxy_list.is_empty() {
                    self.clear_logs(cx);
                    self.add_log(cx, "✗ No enabled proxies");
                    self.add_log(cx, "  Enable at least one proxy");
                    return;
                }

                let interval_secs = interval_minutes * 60;
                let proxy_count = proxy_list.len();

                // Spawn background task
                let handle = std::thread::spawn(move || {
                    use clash_chain_patcher::health::ProxyValidator;
                    use std::thread;
                    use std::time::Duration;

                    let validator = ProxyValidator::new(10);

                    eprintln!("DEBUG: Auto check background thread started, checking every {} minutes", interval_minutes);

                    loop {
                        eprintln!("DEBUG: Starting auto health check cycle");

                        for (proxy_id, host, port, username, password) in &proxy_list {
                            let result = validator.validate(
                                host,
                                *port,
                                username.as_deref(),
                                password.as_deref(),
                            );

                            // Send result to GUI thread
                            if tx.send((proxy_id.clone(), result)).is_err() {
                                eprintln!("DEBUG: Channel closed, stopping auto check");
                                return;
                            }
                        }

                        eprintln!("DEBUG: Auto check cycle completed, sleeping for {} seconds", interval_secs);

                        // Sleep until next check
                        thread::sleep(Duration::from_secs(interval_secs));
                    }
                });

                self.state.auto_checking = true;
                self.ui.button(id!(auto_check_btn)).set_text(cx, "Auto: ON");

                self.clear_logs(cx);
                self.add_log(cx, "✓ Auto health check started");
                self.add_log(cx, &format!("  Checking every {} minutes", interval_minutes));
                self.add_log(cx, &format!("  Monitoring {} enabled proxies", proxy_count));

                eprintln!("DEBUG: Auto check started with {} minute interval", interval_minutes);

                // Note: handle is dropped but thread continues running
                // It will stop when channel is closed
                drop(handle);
            } else {
                self.clear_logs(cx);
                self.add_log(cx, "✗ ProxyState not initialized");
            }
        }

        self.ui.redraw(cx);
    }

    fn update_proxy_health_from_background(
        &mut self,
        cx: &mut Cx,
        proxy_id: String,
        result: clash_chain_patcher::health::ProxyValidationResult,
    ) {
        eprintln!("DEBUG: Received health check result for proxy {}: valid={}", proxy_id, result.is_valid);

        if let Some(state) = &mut self.state.proxy_state {
            if let Some(mut proxy) = state.get_upstream(&proxy_id) {
                if result.is_valid {
                    if let Some(latency) = result.latency_ms {
                        proxy.health.mark_healthy_with_details(
                            latency as u64,
                            result.exit_ip,
                            result.location.as_ref().map(|l| l.format_short()),
                            result.location.as_ref().map(|l| l.country_code.clone()),
                        );
                    }
                } else if let Some(error) = result.error {
                    proxy.health.mark_unhealthy(error);
                }

                let _ = state.update_upstream(proxy);
            }
        }

        // Refresh display
        self.refresh_proxy_list_display(cx);
        self.ui.redraw(cx);
    }

    fn refresh_proxy_list_display(&mut self, cx: &mut Cx) {
        if let Some(state) = &self.state.proxy_state {
            let proxies = state.list_upstreams();
            eprintln!("DEBUG: refresh_proxy_list_display called with {} proxies", proxies.len());

            let enabled_count = proxies.iter().filter(|p| p.enabled).count();
            let healthy_count = proxies.iter()
                .filter(|p| p.enabled && p.health.is_healthy())
                .count();

            // Update stats
            let stats_text = format!(
                "{} proxies, {} enabled, {} healthy",
                proxies.len(),
                enabled_count,
                healthy_count
            );
            eprintln!("DEBUG: Setting stats_text = {}", stats_text);
            self.ui.label(id!(pool_stats_label)).set_text(cx, &stats_text);

            // Update slots (max 10)
            for slot in 0..10 {
                let slot_view_id = match slot {
                    0 => id!(proxy_slot_1), 1 => id!(proxy_slot_2), 2 => id!(proxy_slot_3),
                    3 => id!(proxy_slot_4), 4 => id!(proxy_slot_5), 5 => id!(proxy_slot_6),
                    6 => id!(proxy_slot_7), 7 => id!(proxy_slot_8), 8 => id!(proxy_slot_9),
                    9 => id!(proxy_slot_10), _ => continue,
                };
                let status_id = match slot {
                    0 => id!(proxy_status_1), 1 => id!(proxy_status_2), 2 => id!(proxy_status_3),
                    3 => id!(proxy_status_4), 4 => id!(proxy_status_5), 5 => id!(proxy_status_6),
                    6 => id!(proxy_status_7), 7 => id!(proxy_status_8), 8 => id!(proxy_status_9),
                    9 => id!(proxy_status_10), _ => continue,
                };
                let name_id = match slot {
                    0 => id!(load_btn_1), 1 => id!(load_btn_2), 2 => id!(load_btn_3),
                    3 => id!(load_btn_4), 4 => id!(load_btn_5), 5 => id!(load_btn_6),
                    6 => id!(load_btn_7), 7 => id!(load_btn_8), 8 => id!(load_btn_9),
                    9 => id!(load_btn_10), _ => continue,
                };
                let info_id = match slot {
                    0 => id!(proxy_info_1), 1 => id!(proxy_info_2), 2 => id!(proxy_info_3),
                    3 => id!(proxy_info_4), 4 => id!(proxy_info_5), 5 => id!(proxy_info_6),
                    6 => id!(proxy_info_7), 7 => id!(proxy_info_8), 8 => id!(proxy_info_9),
                    9 => id!(proxy_info_10), _ => continue,
                };

                if let Some(proxy) = proxies.get(slot) {
                    // Show slot
                    self.ui.view(slot_view_id).set_visible(cx, true);

                    // Update status icon (text instead of emoji)
                    let status_icon = if proxy.health.is_healthy() {
                        "✓"  // Green check mark (text)
                    } else if proxy.health.error.is_some() {
                        "×"  // Red X mark (text)
                    } else {
                        "○"  // Gray circle (text)
                    };
                    self.ui.label(status_id).set_text(cx, status_icon);

                    // Update name (as button for loading to form)
                    self.ui.button(name_id).set_text(cx, &proxy.name);

                    // Update info (host:port, latency, location)
                    let mut info_parts = vec![format!("{}:{}", proxy.config.host, proxy.config.port)];

                    if let Some(latency) = proxy.health.latency_ms {
                        info_parts.push(format!("{}ms", latency));
                    }

                    if let Some(location) = &proxy.health.location {
                        info_parts.push(location.clone());
                    } else if let Some(exit_ip) = &proxy.health.exit_ip {
                        info_parts.push(exit_ip.clone());
                    }

                    let info_text = info_parts.join(" | ");
                    self.ui.label(info_id).set_text(cx, &info_text);
                } else {
                    // Hide slot
                    self.ui.view(slot_view_id).set_visible(cx, false);
                }
            }

            // Show/hide empty message
            let empty_visible = proxies.is_empty();
            self.ui.view(id!(proxy_empty_msg)).set_visible(cx, empty_visible);

            // Clear old logs and show current proxy pool state
            self.clear_logs(cx);

            if !proxies.is_empty() {
                self.add_log(cx, "=== Proxy Pool ===");
                for (i, proxy) in proxies.iter().enumerate() {
                    let status_icon = if proxy.health.is_healthy() {
                        "✓"  // Green check mark
                    } else if proxy.health.error.is_some() {
                        "×"  // Red X mark
                    } else {
                        "○"  // Gray circle
                    };
                    let latency_str = if let Some(latency) = proxy.health.latency_ms {
                        format!(" {}ms", latency)
                    } else {
                        String::new()
                    };
                    let enabled_str = if proxy.enabled { "[ON]" } else { "[OFF]" };
                    let mut log_line = format!(
                        "{}. {} {} {}{}",
                        i + 1,
                        status_icon,
                        enabled_str,
                        proxy.name,  // Already contains host:port
                        latency_str
                    );

                    // Add location if available
                    if let Some(location) = &proxy.health.location {
                        log_line.push_str(&format!(" [{}]", location));
                    } else if let Some(exit_ip) = &proxy.health.exit_ip {
                        log_line.push_str(&format!(" [IP: {}]", exit_ip));
                    }

                    self.add_log(cx, &log_line);

                    // Also show error if present
                    if let Some(err) = &proxy.health.error {
                        self.add_log(cx, &format!("   Error: {}", err));
                    }
                }
            } else {
                self.add_log(cx, "Ready");
            }
        }
    }

    fn clear_all_proxies(&mut self, cx: &mut Cx) {
        if let Some(state) = &mut self.state.proxy_state {
            let proxies = state.list_upstreams();
            let count = proxies.len();

            if count == 0 {
                self.clear_logs(cx);
                self.add_log(cx, "No proxies to clear");
                self.ui.redraw(cx);
                return;
            }

            // Remove all proxies
            for proxy in proxies {
                let _ = state.remove_upstream(&proxy.id);
            }

            // Refresh will clear logs and show empty state
            self.refresh_proxy_list_display(cx);
        }
        self.ui.redraw(cx);
    }

    /// Check health of proxy in a specific slot
    fn check_proxy_by_slot(&mut self, cx: &mut Cx, slot_index: usize) {
        // First, get proxy info from state
        let (proxy_id, proxy_name, host, port, username, password) = {
            if let Some(state) = &self.state.proxy_state {
                let proxies = state.list_upstreams();
                if let Some(proxy) = proxies.get(slot_index) {
                    (
                        proxy.id.clone(),
                        proxy.name.clone(),
                        proxy.config.host.clone(),
                        proxy.config.port,
                        proxy.config.username.clone(),
                        proxy.config.password.clone(),
                    )
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        // Now we can use self mutably
        self.clear_logs(cx);
        self.add_log(cx, &format!("Checking {}...", proxy_name));
        self.ui.redraw(cx);

        // Use enhanced validator
        use clash_chain_patcher::health::ProxyValidator;
        let validator = ProxyValidator::new(10);
        let result = validator.validate(
            &host,
            port,
            username.as_deref(),
            password.as_deref(),
        );

        // Update proxy health
        if let Some(state) = &mut self.state.proxy_state {
            if let Some(mut updated_proxy) = state.get_upstream(&proxy_id) {
                if result.is_valid {
                    if let Some(latency) = result.latency_ms {
                        updated_proxy.health.mark_healthy_with_details(
                            latency as u64,
                            result.exit_ip,
                            result.location.as_ref().map(|l| l.format_short()),
                            result.location.as_ref().map(|l| l.country_code.clone()),
                        );
                    }
                } else if let Some(error) = result.error {
                    updated_proxy.health.mark_unhealthy(error);
                }
                let _ = state.update_upstream(updated_proxy);
            }
        }

        self.refresh_proxy_list_display(cx);
        self.ui.redraw(cx);
    }

    /// Load proxy from slot to form (for editing)
    fn load_proxy_to_form(&mut self, cx: &mut Cx, slot_index: usize) {
        if let Some(state) = &self.state.proxy_state {
            let proxies = state.list_upstreams();
            if let Some(proxy) = proxies.get(slot_index) {
                // Load proxy info to form
                self.ui.text_input(id!(host_input)).set_text(cx, &proxy.config.host);
                self.ui.text_input(id!(port_input)).set_text(cx, &proxy.config.port.to_string());

                if let Some(username) = &proxy.config.username {
                    self.ui.text_input(id!(username_input)).set_text(cx, username);
                } else {
                    self.ui.text_input(id!(username_input)).set_text(cx, "");
                }

                if let Some(password) = &proxy.config.password {
                    self.ui.text_input(id!(password_input)).set_text(cx, password);
                } else {
                    self.ui.text_input(id!(password_input)).set_text(cx, "");
                }

                self.clear_logs(cx);
                self.add_log(cx, &format!("✓ Loaded {} to form", proxy.name));
                if proxy.config.username.is_some() {
                    self.add_log(cx, "   (Credentials loaded)");
                }
                self.ui.redraw(cx);
            } else {
                self.clear_logs(cx);
                self.add_log(cx, &format!("✗ Slot {} not found", slot_index + 1));
                self.ui.redraw(cx);
            }
        }
    }

    /// Delete proxy from a specific slot
    fn delete_proxy_by_slot(&mut self, cx: &mut Cx, slot_index: usize) {
        if let Some(state) = &mut self.state.proxy_state {
            let proxies = state.list_upstreams();
            if let Some(proxy) = proxies.get(slot_index) {
                let proxy_id = proxy.id.clone();
                let proxy_name = proxy.name.clone();

                match state.remove_upstream(&proxy_id) {
                    Ok(_) => {
                        self.clear_logs(cx);
                        self.add_log(cx, &format!("✓ Deleted {}", proxy_name));
                        self.refresh_proxy_list_display(cx);
                    }
                    Err(e) => {
                        self.add_log(cx, &format!("✗ Delete error: {}", e));
                    }
                }
                self.ui.redraw(cx);
            }
        }
    }
}
