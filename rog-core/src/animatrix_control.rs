const INIT_STR: &str = "ASUS Tech.Inc.";
const PACKET_SIZE: usize = 640;

// Only these two packets must be 17 bytes
const DEV_PAGE: u8 = 0x5e;
// These bytes are in [1] position of the array
const WRITE: u8 = 0xc0;
const INIT: u8 = 0xc2;
const APPLY: u8 = 0xc3;
const SET: u8 = 0xc4;

use log::{error, warn};
use rog_client::error::AuraError;
use rusb::DeviceHandle;
use std::error::Error;
use std::time::Duration;

#[allow(dead_code)]
#[derive(Debug)]
pub enum AnimatrixCommand {
    Apply,
    Set,
    WriteImage(Vec<Vec<u8>>),
    //ReloadLast,
}

pub struct AniMeWriter {
    handle: DeviceHandle<rusb::GlobalContext>,
    initialised: bool,
}

impl AniMeWriter {
    #[inline]
    pub fn new() -> Result<AniMeWriter, Box<dyn Error>> {
        // We don't expect this ID to ever change
        let mut dev_handle = AniMeWriter::get_device(0x0b05, 0x193b).map_err(|err| {
            warn!("Could not get AniMe display handle: {:?}", err);
            err
        })?;
        dev_handle.reset()?;
        // This config seems to be the required device config for writing
        dev_handle.set_active_configuration(1).unwrap_or(());

        // For the animatrix device there is only one interface and one endpoint
        let interface = 0;

        dev_handle
            .set_auto_detach_kernel_driver(true)
            .map_err(|err| {
                error!("Auto-detach kernel driver failed: {:?}", err);
                err
            })?;
        dev_handle.claim_interface(interface).map_err(|err| {
            error!("Could not claim device interface: {:?}", err);
            err
        })?;

        Ok(AniMeWriter {
            handle: dev_handle,
            initialised: false,
        })
    }

    pub async fn do_command(&mut self, command: AnimatrixCommand) -> Result<(), AuraError> {
        if !self.initialised {
            self.do_initialization().await?
        }

        match command {
            AnimatrixCommand::WriteImage(effect) => self.write_image(effect).await?,
            AnimatrixCommand::Set => self.do_set().await?,
            AnimatrixCommand::Apply => self.do_apply().await?,
            //AnimatrixCommand::ReloadLast => self.reload_last_builtin(&config).await?,
        }
        Ok(())
    }

    /// Should only be used if the bytes you are writing are verified correct
    #[inline]
    async fn write_bytes(&self, message: &[u8]) -> Result<(), AuraError> {
        let prev = std::time::Instant::now();
        match self.handle.write_control(
            0x21,  // request_type
            0x09,  // request
            0x35e, // value
            0x00,  // index
            message,
            Duration::from_millis(200),
        ) {
            Ok(_) => {
                println!(
                    "Written in {:?} us",
                    std::time::Instant::now().duration_since(prev).as_micros()
                );
            }
            Err(err) => match err {
                rusb::Error::Timeout => {}
                _ => error!("Failed to write to led interrupt: {:?}", err),
            },
        }
        Ok(())
    }

    /// Write an Animatrix image
    ///
    /// The expected input here is *two* Vectors, 640 bytes in length. The two vectors
    /// are each one half of the full image write.
    ///
    /// After each write a flush is written, it is assumed that this tells the device to
    /// go ahead and display the written bytes
    ///
    /// # Note:
    /// The vectors are expected to contain the full sequence of bytes as follows
    ///
    /// - Write pane 1: 0x5e 0xc0 0x02 0x01 0x00 0x73 0x02 .. <led brightness>
    /// - Write pane 2: 0x5e 0xc0 0x02 0x74 0x02 0x73 0x02 .. <led brightness>
    ///
    /// Where led brightness is 0..255, low to high
    #[inline]
    async fn write_image(&mut self, image: Vec<Vec<u8>>) -> Result<(), AuraError> {
        for row in image.iter() {
            self.write_bytes(row).await?;
        }
        self.do_flush().await?;
        Ok(())
    }

    // TODO: save/reload last frame
    #[inline]
    fn get_device(
        vendor: u16,
        product: u16,
    ) -> Result<DeviceHandle<rusb::GlobalContext>, rusb::Error> {
        for device in rusb::devices()?.iter() {
            let device_desc = device.device_descriptor()?;
            if device_desc.vendor_id() == vendor && device_desc.product_id() == product {
                return device.open();
            }
        }
        Err(rusb::Error::NoDevice)
    }

    #[inline]
    async fn do_initialization(&mut self) -> Result<(), AuraError> {
        let mut init = [0; PACKET_SIZE];
        init[0] = DEV_PAGE; // This is the USB page we're using throughout
        for (idx, byte) in INIT_STR.as_bytes().iter().enumerate() {
            init[idx + 1] = *byte
        }
        self.write_bytes(&init).await?;
        // clear the init array and write other init message
        for idx in 0..INIT_STR.len() {
            match idx {
                0 => init[idx] = DEV_PAGE, // write it to be sure?
                1 => init[idx] = INIT,
                _ => init[idx] = 0,
            }
        }
        self.write_bytes(&init).await?;
        self.initialised = true;
        Ok(())
    }

    #[inline]
    async fn do_flush(&mut self) -> Result<(), AuraError> {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = WRITE;
        flush[2] = 0x03;

        self.write_bytes(&flush).await?;
        Ok(())
    }

    #[inline]
    async fn do_set(&mut self) -> Result<(), AuraError> {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = SET;
        flush[2] = 0x01;
        flush[3] = 0x80;

        self.write_bytes(&flush).await?;
        Ok(())
    }

    #[inline]
    async fn do_apply(&mut self) -> Result<(), AuraError> {
        let mut flush = [0; PACKET_SIZE];
        flush[0] = DEV_PAGE;
        flush[1] = APPLY;
        flush[2] = 0x01;
        flush[3] = 0x80;

        self.write_bytes(&flush).await?;
        Ok(())
    }
}
