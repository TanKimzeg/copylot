pub mod windows {
    use arboard::Clipboard;
    use log;
    use std::error::Error;
    use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, IUIAutomationTextPattern, UIA_TextPatternId,
    };

    pub async fn get_selected_text() -> String {
        if let Ok(res) = tauri::async_runtime::spawn_blocking(|| get_text_by_clipboard()).await {
            match res {
                Ok(text) if !text.is_empty() => return text,
                Ok(_) => log::info!("get_text_by_clipboard is empty"),
                Err(err) => log::error!("get_text_by_clipboard error:{}", err),
            }
        }

        log::info!("fallback to get_text_by_automation");

        // automation 放到阻塞线程，避免卡住事件/消息相关线程
        if let Ok(res) = tauri::async_runtime::spawn_blocking(|| get_text_by_automation()).await {
            match res {
                Ok(text) if !text.is_empty() => return text,
                Ok(_) => log::info!("get_text_by_automation is empty"),
                Err(err) => log::error!("get_text_by_automation error:{}", err),
            }
        }
        String::new()
    }

    /// Available for almost all applications
    fn get_text_by_clipboard() -> Result<String, Box<dyn Error + Send + Sync>> {
        // Read Old Clipboard
        let old_clipboard = (Clipboard::new()?.get_text(), Clipboard::new()?.get_image());
        let timeout = std::time::Duration::from_millis(500);

        if copy(timeout) {
            // Read New Clipboard
            let new_text = Clipboard::new()?.get_text();

            // Create Write Clipboard
            let mut write_clipboard = Clipboard::new()?;

            match old_clipboard {
                (Ok(text), _) => {
                    // Old Clipboard is Text
                    write_clipboard.set_text(text)?;
                    if let Ok(new) = new_text {
                        Ok(new.trim().to_string())
                    } else {
                        Err("New clipboard is not Text".into())
                    }
                }
                (_, Ok(image)) => {
                    // Old Clipboard is Image
                    write_clipboard.set_image(image)?;
                    if let Ok(new) = new_text {
                        Ok(new.trim().to_string())
                    } else {
                        Err("New clipboard is not Text".into())
                    }
                }
                _ => {
                    // Old Clipboard is Empty
                    write_clipboard.clear()?;
                    if let Ok(new) = new_text {
                        Ok(new.trim().to_string())
                    } else {
                        Err("New clipboard is not Text".into())
                    }
                }
            }
        } else {
            Err("Copy Failed".into())
        }
    }

    fn copy(timeout: std::time::Duration) -> bool {
        use enigo::{
            Direction::{Click, Press, Release},
            Enigo, Key, Keyboard, Settings,
        };

        let num_before = unsafe { GetClipboardSequenceNumber() };

        let mut enigo = match Enigo::new(&Settings::default()) {
            Ok(e) => e,
            Err(e) => {
                log::error!("Enigo::new failed: {e:?}");
                return false;
            }
        };

        // 清理修饰键，避免卡键
        let _ = enigo.key(Key::Alt, Release);
        let _ = enigo.key(Key::Shift, Release);
        let _ = enigo.key(Key::Meta, Release);
        let _ = enigo.key(Key::Escape, Release);
        let _ = enigo.key(Key::Control, Release);

        // 发送 Ctrl + c（用 Unicode 更稳，避免布局/键码问题）
        if enigo.key(Key::Control, Press).is_err() {
            return false;
        }
        let _ = enigo.key(Key::C, Click);
        let _ = enigo.key(Key::Control, Release);

        // 等待剪贴板序列号变化（最多 timeout）
        let start = std::time::Instant::now();
        loop {
            let num_after = unsafe { GetClipboardSequenceNumber() };
            if num_after != num_before {
                log::info!("num_before: {}, num_after: {}", num_before, num_after);
                return true;
            }
            if start.elapsed() >= timeout {
                log::info!("num_before: {}, num_after: {}", num_before, num_after);
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// 获取当前焦点窗口中选中的文本（UI Automation 尝试）
    /// Available for Edge, Chrome and UWP
    fn get_text_by_automation() -> Result<String, Box<dyn Error + Send + Sync>> {
        // see: https://github.com/pot-app/Selection/blob/622e8f32c851daabe45c88481a1c59d11bc7256e/src/windows.rs#L49
        // see: https://github.com/any-menu/any-menu/blob/b70068ba4eb36effdfe895cbad7425e01ffc5419/src/Tauri/src-tauri/src/uia.rs#L865
        use windows::core::Interface;
        use windows::Win32::System::Com::*;
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            let ui_automation: IUIAutomation =
                match CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) {
                    Ok(ua) => ua,
                    Err(_) => {
                        CoUninitialize();
                        return Err("获取UI自动化实例失败".into());
                    }
                };

            // 获取焦点元素 (可靠)
            let element = match ui_automation.GetFocusedElement() {
                Ok(element) => element,
                Err(_) => {
                    CoUninitialize();
                    return Err("获取焦点元素失败".into());
                }
            };

            // 文本模式 (Text Pattern)，以获取选中内容和插入符位置 (可靠，Ndd常失灵)
            let text_pattern = match element.GetCurrentPattern(UIA_TextPatternId) {
                Ok(tp) => tp,
                Err(_) => {
                    CoUninitialize();
                    return Err("获取文本模式失败".into());
                }
            };

            // 类型转换
            let text_pattern: IUIAutomationTextPattern = match text_pattern.cast() {
                Ok(tp) => tp,
                Err(_) => {
                    CoUninitialize();
                    return Err("文本模式转换失败".into());
                }
            };

            // 获取选中的文本范围 (似乎不一定支持多光标，识别的是主光标区域)
            if let Ok(selection) = text_pattern.GetSelection() {
                let count = match selection.Length() {
                    Ok(c) => c,
                    Err(_) => {
                        CoUninitialize();
                        return Err("获取选中文本范围失败".into());
                    }
                };

                for i in 0..count {
                    if let Ok(range) = selection.GetElement(i) {
                        if let Ok(text) = range.GetText(-1) {
                            return Ok(text.to_string());
                        }
                    }
                }
            }
            CoUninitialize();
            return Ok("".into()); // 没有选中文本
        }
    }
}
