#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]

extern crate embassy_rp as emb_rp;

mod register;

const DEFAULT_ADDRESS: u8 = 0x76;

pub struct BMP280<'d> {
    i2c: emb_rp::i2c::I2c<'d, emb_rp::peripherals::I2C1, emb_rp::i2c::Async>,
}

impl<'d> BMP280<'d> {
    pub fn new(i2c: emb_rp::i2c::I2c<'d, emb_rp::peripherals::I2C1, emb_rp::i2c::Async>) -> Self {
        Self { i2c }
    }

    pub async fn read_id(&mut self) -> (Result<u8, emb_rp::i2c::Error>) {
        let mut buffer: [u8; 1] = [0; 1];

        let id: u8 = register::Register::ID.addr();
        self.i2c
            .write_read_async(DEFAULT_ADDRESS, [id], &mut buffer)
            .await
            .unwrap();

        Ok(buffer[0])
    }

    pub async fn configure_sensor(&mut self) -> Result<(), emb_rp::i2c::Error> {
        let ctrl_meas: u8 = 0b00100111; // Temp oversampling x1, Pressure oversampling x1, Normal mode
        let config: u8 = 0b10100000; // Standby time 1000ms, filter off

        self.i2c
            .write_async(DEFAULT_ADDRESS, [0xF4, ctrl_meas])
            .await?;
        self.i2c
            .write_async(DEFAULT_ADDRESS, [0xF5, config])
            .await?;

        log::info!(
            "BMP280 configured: CTRL_MEAS=0x{:X}, CONFIG=0x{:X}",
            ctrl_meas,
            config
        );

        Ok(())
    }

    pub async fn read_calibration(&mut self) -> Result<(u16, i16, i16), emb_rp::i2c::Error> {
        let mut calib_data: [u8; 24] = [0u8; 24];
        self.i2c
            .write_read_async(
                DEFAULT_ADDRESS,
                [register::Register::CALIB.addr()],
                &mut calib_data,
            )
            .await?;

        let dig_t1 = ((calib_data[1] as u16) << 8) | (calib_data[0] as u16);
        let dig_t2 = ((calib_data[3] as i16) << 8) | (calib_data[2] as i16);
        let dig_t3 = ((calib_data[5] as i16) << 8) | (calib_data[4] as i16);

        Ok((dig_t1, dig_t2, dig_t3))
    }

    pub async fn read_temperature(
        &mut self,
        dig_T1: u16,
        dig_T2: i16,
        dig_T3: i16,
    ) -> Result<f32, emb_rp::i2c::Error> {
        let mut temp_raw = [0u8; 3];
        self.i2c
            .write_read_async(
                DEFAULT_ADDRESS,
                [register::Register::TEMPERATURE.addr()],
                &mut temp_raw,
            )
            .await?;

        let adc_T = ((temp_raw[0] as i32) << 12)
            | ((temp_raw[1] as i32) << 4)
            | ((temp_raw[2] as i32) >> 4);

        let var1 = (((adc_T >> 3) - ((dig_T1 as i32) << 1)) * (dig_T2 as i32)) >> 11;
        let var2 = (((((adc_T >> 4) - (dig_T1 as i32)) * ((adc_T >> 4) - (dig_T1 as i32))) >> 12)
            * (dig_T3 as i32))
            >> 14;

        let t_fine = var1 + var2;
        let temperature = ((t_fine * 5 + 128) >> 8) as f32 / 100.0;

        Ok(temperature)
    }
}
