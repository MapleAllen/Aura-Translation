use serde::Serialize;

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum FeatureCapability {
    Ready,
    Unsupported,
}

#[derive(Serialize, Clone, Copy, Debug)]
pub struct SystemCapabilities {
    pub aura_mode: FeatureCapability,
}

#[tauri::command]
pub fn get_system_capabilities() -> SystemCapabilities {
    #[cfg(target_os = "windows")]
    {
        SystemCapabilities {
            aura_mode: FeatureCapability::Ready,
        }
    }

    #[cfg(target_os = "macos")]
    {
        SystemCapabilities {
            aura_mode: FeatureCapability::Ready,
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        SystemCapabilities {
            aura_mode: FeatureCapability::Unsupported,
        }
    }
}
