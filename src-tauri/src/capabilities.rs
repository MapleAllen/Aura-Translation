use serde::Serialize;

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum FeatureCapability {
    Ready,
    NeedsPermission,
    Unsupported,
}

#[derive(Serialize, Clone, Copy, Debug)]
pub struct SystemCapabilities {
    pub aura_mode: FeatureCapability,
    pub paste_back: FeatureCapability,
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

#[tauri::command]
pub fn get_system_capabilities() -> SystemCapabilities {
    #[cfg(target_os = "windows")]
    {
        SystemCapabilities {
            aura_mode: FeatureCapability::Ready,
            paste_back: FeatureCapability::Ready,
        }
    }

    #[cfg(target_os = "macos")]
    {
        let is_trusted = unsafe { AXIsProcessTrusted() };
        let paste_back = if is_trusted {
            FeatureCapability::Ready
        } else {
            FeatureCapability::NeedsPermission
        };

        SystemCapabilities {
            aura_mode: FeatureCapability::Ready,
            paste_back,
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        SystemCapabilities {
            aura_mode: FeatureCapability::Unsupported,
            paste_back: FeatureCapability::Unsupported,
        }
    }
}
