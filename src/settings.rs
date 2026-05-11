use ledger_device_sdk::nvm::*;
use ledger_device_sdk::NVMData;

/// NVM settings layout:
/// [0] = blind signing enabled (0 = disabled, 1 = enabled)
/// [1..9] = reserved
const SETTINGS_SIZE: usize = 10;

/// Index for blind signing toggle in NVM settings.
pub const BLIND_SIGNING_INDEX: usize = 0;

#[link_section = ".nvm_data"]
static mut DATA: NVMData<AtomicStorage<[u8; SETTINGS_SIZE]>> =
    NVMData::new(AtomicStorage::new(&[0u8; SETTINGS_SIZE]));

#[derive(Clone, Copy)]
pub struct Settings;

impl Default for Settings {
    fn default() -> Self {
        Settings
    }
}

impl Settings {
    #[inline(never)]
    pub fn get_mut(&mut self) -> &mut AtomicStorage<[u8; SETTINGS_SIZE]> {
        let data = &raw mut DATA;
        unsafe { (*data).get_mut() }
    }

    pub fn get_element(&self, index: usize) -> u8 {
        let data = &raw const DATA;
        let storage = unsafe { (*data).get_ref() };
        let settings = storage.get_ref();
        settings[index]
    }

    #[allow(dead_code)]
    pub fn set_element(&self, index: usize, value: u8) {
        let data = &raw mut DATA;
        let storage = unsafe { (*data).get_mut() };
        let mut updated_data = *storage.get_ref();
        updated_data[index] = value;
        storage.update(&updated_data);
    }

    pub fn blind_signing_enabled(&self) -> bool {
        self.get_element(BLIND_SIGNING_INDEX) != 0
    }
}
