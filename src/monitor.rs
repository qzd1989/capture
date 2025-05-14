use anyhow::{Result, anyhow};
use display_info::DisplayInfo;
pub enum Monitor {
    Primary,
}
impl Monitor {
    pub fn info(&self) -> Result<DisplayInfo> {
        match self {
            Monitor::Primary => {
                let display_infos = DisplayInfo::all().unwrap();
                for display_info in display_infos {
                    if display_info.is_primary {
                        return Ok(display_info);
                    }
                }
                Err(anyhow!("No primary display found"))
            }
        }
    }
    pub fn id(&self) -> Result<u32> {
        Ok(self.info()?.id)
    }
    // pub fn all() -> Result<Vec<DisplayInfo>> {
    //     DisplayInfo::all().map_err(|error| anyhow!(error))
    // }
}
