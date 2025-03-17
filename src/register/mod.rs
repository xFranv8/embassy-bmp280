pub enum Register {
    ID,
    CALIB,
    // PRESSURE,
    TEMPERATURE,
}

impl Register {
    pub fn addr(&self) -> u8 {
        match self {
            Register::ID => 0xD0,
            Register::CALIB => 0x88,
            // Register::PRESSURE => 0xF7,
            Register::TEMPERATURE => 0xFA,
        }
    }
}
