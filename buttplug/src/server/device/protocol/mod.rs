// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

//! Implementations of communication protocols for hardware supported by Buttplug

pub mod generic_command_manager;

// Utility mods
pub mod fleshlight_launch_helper;

// Since users can pick and choose protocols, we need all of these to be public.
pub mod aneros;
pub mod ankni;
pub mod buttplug_passthru;
pub mod cachito;
pub mod fredorch;
pub mod hgod;
pub mod hismith;
pub mod htk_bm;
pub mod jejoue;
pub mod kgoal_boost;
pub mod kiiroo_v2;
pub mod kiiroo_v21;
pub mod kiiroo_v21_initialized;
pub mod kiiroo_v2_vibrator;
pub mod lelof1s;
pub mod lelof1sv2;
pub mod libo_elle;
pub mod libo_shark;
pub mod libo_vibes;
pub mod lovedistance;
pub mod lovehoney_desire;
pub mod lovense;
pub mod lovense_connect_service;
pub mod lovenuts;
pub mod magic_motion_v1;
pub mod magic_motion_v2;
pub mod magic_motion_v3;
pub mod magic_motion_v4;
pub mod mannuo;
pub mod maxpro;
pub mod mizzzee;
pub mod motorbunny;
pub mod mysteryvibe;
pub mod nobra;
pub mod patoo;
pub mod picobong;
pub mod prettylove;
pub mod raw_protocol;
pub mod realov;
pub mod satisfyer;
pub mod svakom;
pub mod svakom_alex;
pub mod svakom_iker;
pub mod svakom_sam;
pub mod tcode_v03;
pub mod thehandy;
pub mod vibratissimo;
pub mod vorze_sa;
pub mod wevibe;
pub mod wevibe8bit;
pub mod xinput;
pub mod youcups;
pub mod youou;
pub mod zalo;

use crate::{
  core::{
    errors::ButtplugDeviceError,
    messages::{
      self,
      ActuatorType,
      ButtplugDeviceCommandMessageUnion,
      ButtplugDeviceMessage,
      ButtplugServerDeviceMessage,
      ButtplugServerMessage,
      Endpoint,
      RawReading,
      SensorType,
    },
  },
  server::device::{
    configuration::{ProtocolAttributesType, ProtocolCommunicationSpecifier},
    hardware::{Hardware, HardwareCommand, HardwareReadCmd},
    ServerDeviceIdentifier,
  },
};
use async_trait::async_trait;
use futures::{
  future::{self, BoxFuture, FutureExt},
  StreamExt,
};
use std::pin::Pin;
use std::{collections::HashMap, sync::Arc};

pub trait ProtocolIdentifierFactory: Send + Sync {
  fn identifier(&self) -> &str;
  fn create(&self) -> Box<dyn ProtocolIdentifier>;
}

pub fn get_default_protocol_map() -> HashMap<String, Arc<dyn ProtocolIdentifierFactory>> {
  let mut map = HashMap::new();
  fn add_to_protocol_map<T>(
    map: &mut HashMap<String, Arc<dyn ProtocolIdentifierFactory>>,
    factory: T,
  ) where
    T: ProtocolIdentifierFactory + 'static,
  {
    let factory = Arc::new(factory);
    map.insert(factory.identifier().to_owned(), factory);
  }

  add_to_protocol_map(&mut map, aneros::setup::AnerosIdentifierFactory::default());
  add_to_protocol_map(
    &mut map,
    buttplug_passthru::setup::ButtplugPassthruIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    cachito::setup::CachitoIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    lovense::setup::LovenseIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    hismith::setup::HismithIdentifierFactory::default(),
  );
  add_to_protocol_map(&mut map, htk_bm::setup::HtkBmIdentifierFactory::default());
  add_to_protocol_map(
    &mut map,
    thehandy::setup::TheHandyIdentifierFactory::default(),
  );

  add_to_protocol_map(&mut map, ankni::setup::AnkniIdentifierFactory::default());
  add_to_protocol_map(
    &mut map,
    fredorch::setup::FredorchIdentifierFactory::default(),
  );

  add_to_protocol_map(&mut map, hgod::setup::HgodIdentifierFactory::default());

  add_to_protocol_map(&mut map, jejoue::setup::JeJoueIdentifierFactory::default());
  add_to_protocol_map(
    &mut map,
    kiiroo_v2::setup::KiirooV2IdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    kiiroo_v2_vibrator::setup::KiirooV2VibratorIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    kiiroo_v21::setup::KiirooV21IdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    kiiroo_v21_initialized::setup::KiirooV21InitializedIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    lelof1s::setup::LeloF1sIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    lelof1sv2::setup::LeloF1sV2IdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    libo_elle::setup::LiboElleIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    libo_shark::setup::LiboSharkIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    libo_vibes::setup::LiboVibesIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    lovehoney_desire::setup::LovehoneyDesireIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    lovedistance::setup::LoveDistanceIdentifierFactory::default(),
  );

  add_to_protocol_map(
    &mut map,
    lovense_connect_service::setup::LovenseConnectServiceIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    lovenuts::setup::LoveNutsIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    magic_motion_v1::setup::MagicMotionV1IdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    magic_motion_v2::setup::MagicMotionV2IdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    magic_motion_v3::setup::MagicMotionV3IdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    magic_motion_v4::setup::MagicMotionV4IdentifierFactory::default(),
  );
  add_to_protocol_map(&mut map, mannuo::setup::ManNuoIdentifierFactory::default());
  add_to_protocol_map(&mut map, maxpro::setup::MaxproIdentifierFactory::default());
  add_to_protocol_map(
    &mut map,
    mizzzee::setup::MizzZeeIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    motorbunny::setup::MotorbunnyIdentifierFactory::default(),
  );
  add_to_protocol_map(&mut map, mysteryvibe::setup::MysteryVibeIdentifierFactory::default());
  add_to_protocol_map(&mut map, nobra::setup::NobraIdentifierFactory::default());
  add_to_protocol_map(&mut map, patoo::setup::PatooIdentifierFactory::default());
  add_to_protocol_map(
    &mut map,
    picobong::setup::PicobongIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    prettylove::setup::PrettyLoveIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    raw_protocol::setup::RawProtocolIdentifierFactory::default(),
  );
  add_to_protocol_map(&mut map, realov::setup::RealovIdentifierFactory::default());
  add_to_protocol_map(&mut map, satisfyer::setup::SatisfyerIdentifierFactory::default());
  add_to_protocol_map(&mut map, svakom::setup::SvakomIdentifierFactory::default());
  add_to_protocol_map(
    &mut map,
    svakom_alex::setup::SvakomAlexIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    svakom_iker::setup::SvakomIkerIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    svakom_sam::setup::SvakomSamIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    tcode_v03::setup::TCodeV03IdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    vibratissimo::setup::VibratissimoIdentifierFactory::default(),
  );
  add_to_protocol_map(
    &mut map,
    vorze_sa::setup::VorzeSAIdentifierFactory::default(),
  );
  add_to_protocol_map(&mut map, wevibe::setup::WeVibeIdentifierFactory::default());
  add_to_protocol_map(
    &mut map,
    wevibe8bit::setup::WeVibe8BitIdentifierFactory::default(),
  );
  add_to_protocol_map(&mut map, xinput::setup::XInputIdentifierFactory::default());
  add_to_protocol_map(
    &mut map,
    youcups::setup::YoucupsIdentifierFactory::default(),
  );
  add_to_protocol_map(&mut map, youou::setup::YououIdentifierFactory::default());
  add_to_protocol_map(&mut map, zalo::setup::ZaloIdentifierFactory::default());
  add_to_protocol_map(
    &mut map,
    kgoal_boost::setup::KGoalBoostIdentifierFactory::default(),
  );
  map
}

fn print_type_of<T>(_: &T) -> &'static str {
  std::any::type_name::<T>()
}

pub struct ProtocolSpecializer {
  specifiers: Vec<ProtocolCommunicationSpecifier>,
  identifier: Box<dyn ProtocolIdentifier>,
}

impl ProtocolSpecializer {
  pub fn new(
    specifiers: Vec<ProtocolCommunicationSpecifier>,
    identifier: Box<dyn ProtocolIdentifier>,
  ) -> Self {
    Self {
      specifiers,
      identifier,
    }
  }

  pub fn specifiers(&self) -> &Vec<ProtocolCommunicationSpecifier> {
    &self.specifiers
  }

  pub fn identify(self) -> Box<dyn ProtocolIdentifier> {
    self.identifier
  }
}

#[async_trait]
pub trait ProtocolIdentifier: Sync + Send {
  async fn identify(
    &mut self,
    hardware: Arc<Hardware>,
  ) -> Result<(ServerDeviceIdentifier, Box<dyn ProtocolInitializer>), ButtplugDeviceError>;
}

#[async_trait]
pub trait ProtocolInitializer: Sync + Send {
  async fn initialize(
    &mut self,
    hardware: Arc<Hardware>,
  ) -> Result<Arc<dyn ProtocolHandler>, ButtplugDeviceError>;
}

pub struct GenericProtocolIdentifier {
  handler: Option<Arc<dyn ProtocolHandler>>,
  protocol_identifier: String,
}

impl GenericProtocolIdentifier {
  pub fn new(handler: Arc<dyn ProtocolHandler>, protocol_identifier: &str) -> Self {
    Self {
      handler: Some(handler),
      protocol_identifier: protocol_identifier.to_owned(),
    }
  }
}

#[async_trait]
impl ProtocolIdentifier for GenericProtocolIdentifier {
  async fn identify(
    &mut self,
    hardware: Arc<Hardware>,
  ) -> Result<(ServerDeviceIdentifier, Box<dyn ProtocolInitializer>), ButtplugDeviceError> {
    let device_identifier = ServerDeviceIdentifier::new(
      hardware.address(),
      &self.protocol_identifier,
      &ProtocolAttributesType::Identifier(hardware.name().to_owned()),
    );
    Ok((
      device_identifier,
      Box::new(GenericProtocolInitializer::new(
        self.handler.take().unwrap(),
      )),
    ))
  }
}

pub struct GenericProtocolInitializer {
  handler: Option<Arc<dyn ProtocolHandler>>,
}

impl GenericProtocolInitializer {
  pub fn new(handler: Arc<dyn ProtocolHandler>) -> Self {
    Self {
      handler: Some(handler),
    }
  }
}

#[async_trait]
impl ProtocolInitializer for GenericProtocolInitializer {
  async fn initialize(
    &mut self,
    _: Arc<Hardware>,
  ) -> Result<Arc<dyn ProtocolHandler>, ButtplugDeviceError> {
    Ok(self.handler.take().unwrap())
  }
}

pub trait ProtocolHandler: Sync + Send {
  fn needs_full_command_set(&self) -> bool {
    false
  }

  fn has_handle_message(&self) -> bool {
    false
  }

  fn handle_message(
    &self,
    message: &ButtplugDeviceCommandMessageUnion,
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    self.command_unimplemented(print_type_of(&message))
  }

  fn command_unimplemented(
    &self,
    command: &str,
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    #[cfg(build = "debug")]
    unimplemented!("Command not implemented for this protocol");
    #[cfg(not(build = "debug"))]
    Err(ButtplugDeviceError::UnhandledCommand(format!(
      "Command not implemented for this protocol: {}",
      command
    )))
  }

  // The default scalar handler assumes that most devices require discrete commands per feature. If
  // a protocol has commands that combine multiple features, either with matched or unmatched
  // actuators, they should just implement their own version of this method.
  fn handle_scalar_cmd(
    &self,
    commands: &[Option<(ActuatorType, u32)>],
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    let mut command_vec = vec![];
    for (index, command) in commands.iter().enumerate().filter(|(_, x)| x.is_some()) {
      let (actuator, scalar) = command.as_ref().expect("Already verified existence");
      command_vec.append(
        &mut (match *actuator {
          ActuatorType::Constrict => self.handle_scalar_constrict_cmd(index as u32, *scalar)?,
          ActuatorType::Inflate => self.handle_scalar_inflate_cmd(index as u32, *scalar)?,
          ActuatorType::Oscillate => self.handle_scalar_oscillate_cmd(index as u32, *scalar)?,
          ActuatorType::Rotate => self.handle_scalar_rotate_cmd(index as u32, *scalar)?,
          ActuatorType::Vibrate => self.handle_scalar_vibrate_cmd(index as u32, *scalar)?,
          ActuatorType::Position => self.handle_scalar_position_cmd(index as u32, *scalar)?,
        }),
      );
    }
    Ok(command_vec)
  }

  fn handle_scalar_vibrate_cmd(
    &self,
    _index: u32,
    _scalar: u32,
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    self.command_unimplemented("ScalarCmd (Vibrate Actuator)")
  }

  fn handle_scalar_rotate_cmd(
    &self,
    _index: u32,
    _scalar: u32,
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    self.command_unimplemented("ScalarCmd (Rotate Actuator)")
  }

  fn handle_scalar_oscillate_cmd(
    &self,
    _index: u32,
    _scalar: u32,
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    self.command_unimplemented("ScalarCmd (Osccilate Actuator)")
  }

  fn handle_scalar_inflate_cmd(
    &self,
    _index: u32,
    _scalar: u32,
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    self.command_unimplemented("ScalarCmd (Inflate Actuator)")
  }

  fn handle_scalar_constrict_cmd(
    &self,
    _index: u32,
    _scalar: u32,
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    self.command_unimplemented("ScalarCmd (Constrict Actuator)")
  }

  fn handle_scalar_position_cmd(
    &self,
    _index: u32,
    _scalar: u32,
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    self.command_unimplemented("ScalarCmd (Constrict Actuator)")
  }

  fn handle_vorze_a10_cyclone_cmd(
    &self,
    message: messages::VorzeA10CycloneCmd,
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    self.command_unimplemented(print_type_of(&message))
  }

  fn handle_kiiroo_cmd(
    &self,
    message: messages::KiirooCmd,
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    self.command_unimplemented(print_type_of(&message))
  }

  fn handle_fleshlight_launch_fw12_cmd(
    &self,
    message: messages::FleshlightLaunchFW12Cmd,
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    self.command_unimplemented(print_type_of(&message))
  }

  fn handle_rotate_cmd(
    &self,
    _commands: &Vec<Option<(u32, bool)>>,
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    self.command_unimplemented("RotateCmd")
  }

  fn handle_linear_cmd(
    &self,
    message: messages::LinearCmd,
  ) -> Result<Vec<HardwareCommand>, ButtplugDeviceError> {
    self.command_unimplemented(print_type_of(&message))
  }

  fn handle_sensor_subscribe_cmd(
    &self,
    _device: Arc<Hardware>,
    _message: messages::SensorSubscribeCmd,
  ) -> BoxFuture<Result<ButtplugServerMessage, ButtplugDeviceError>> {
    future::ready(Err(ButtplugDeviceError::UnhandledCommand(
      "Command not implemented for this protocol: BatteryCmd".to_string(),
    )))
    .boxed()
  }

  fn handle_sensor_unsubscribe_cmd(
    &self,
    _device: Arc<Hardware>,
    _message: messages::SensorUnsubscribeCmd,
  ) -> BoxFuture<Result<ButtplugServerMessage, ButtplugDeviceError>> {
    future::ready(Err(ButtplugDeviceError::UnhandledCommand(
      "Command not implemented for this protocol: BatteryCmd".to_string(),
    )))
    .boxed()
  }

  fn handle_sensor_read_cmd(
    &self,
    device: Arc<Hardware>,
    message: messages::SensorReadCmd,
  ) -> BoxFuture<Result<ButtplugServerMessage, ButtplugDeviceError>> {
    match message.sensor_type() {
      SensorType::Battery => self.handle_battery_level_cmd(device, message),
      _ => future::ready(Err(ButtplugDeviceError::UnhandledCommand(
        "Command not implemented for this protocol: SensorReadCmd".to_string(),
      )))
      .boxed(),
    }
  }

  fn handle_battery_level_cmd(
    &self,
    device: Arc<Hardware>,
    message: messages::SensorReadCmd,
  ) -> BoxFuture<Result<ButtplugServerMessage, ButtplugDeviceError>> {
    // If we have a standardized BLE Battery endpoint, handle that above the
    // protocol, as it'll always be the same.
    if device.endpoints().contains(&Endpoint::RxBLEBattery) {
      debug!("Trying to get battery reading.");
      let msg = HardwareReadCmd::new(Endpoint::RxBLEBattery, 1, 0);
      let fut = device.read_value(&msg);
      async move {
        let raw_msg: RawReading = fut.await?;
        let battery_level = raw_msg.data()[0] as i32;
        let battery_reading = messages::SensorReading::new(
          message.device_index(),
          *message.sensor_index(),
          *message.sensor_type(),
          vec![battery_level],
        );
        debug!("Got battery reading: {}", battery_level);
        Ok(battery_reading.into())
      }
      .boxed()
    } else {
      future::ready(Err(ButtplugDeviceError::UnhandledCommand(
        "Command not implemented for this protocol: SensorReadCmd".to_string(),
      )))
      .boxed()
    }
  }

  fn handle_rssi_level_cmd(
    &self,
    _device: Arc<Hardware>,
    _message: messages::RSSILevelCmd,
  ) -> BoxFuture<Result<ButtplugServerMessage, ButtplugDeviceError>> {
    future::ready(Err(ButtplugDeviceError::UnhandledCommand(
      "Command not implemented for this protocol: SensorReadCmd".to_string(),
    )))
    .boxed()
  }

  fn event_stream(
    &self,
  ) -> Pin<Box<dyn tokio_stream::Stream<Item = ButtplugServerDeviceMessage> + Send>> {
    tokio_stream::empty().boxed()
  }
}

#[macro_export]
macro_rules! generic_protocol_setup {
  ( $protocol_name:ident, $protocol_identifier:tt) => {
    paste::paste! {
      pub mod setup {
        use std::sync::Arc;
        use $crate::server::device::protocol::{
          GenericProtocolIdentifier, ProtocolIdentifier, ProtocolIdentifierFactory,
        };
        #[derive(Default)]
        pub struct [< $protocol_name IdentifierFactory >] {}

        impl ProtocolIdentifierFactory for  [< $protocol_name IdentifierFactory >] {
          fn identifier(&self) -> &str {
            $protocol_identifier
          }

          fn create(&self) -> Box<dyn ProtocolIdentifier> {
            Box::new(GenericProtocolIdentifier::new(
              Arc::new(super::$protocol_name::default()),
              self.identifier(),
            ))
          }
        }
      }
    }
  };
}

#[macro_export]
macro_rules! generic_protocol_initializer_setup {
  ( $protocol_name:ident, $protocol_identifier:tt) => {
    paste::paste! {
      pub mod setup {
        use $crate::server::device::protocol::{ProtocolIdentifier, ProtocolIdentifierFactory};
        #[derive(Default)]
        pub struct [< $protocol_name IdentifierFactory >] {}

        impl ProtocolIdentifierFactory for [< $protocol_name IdentifierFactory >] {
          fn identifier(&self) -> &str {
            $protocol_identifier
          }

          fn create(&self) -> Box<dyn ProtocolIdentifier> {
            Box::new(super::[< $protocol_name Identifier >]::default())
          }
        }
      }

      #[derive(Default)]
      pub struct [< $protocol_name Identifier >] {}

      #[async_trait]
      impl ProtocolIdentifier for [< $protocol_name Identifier >] {
        async fn identify(
          &mut self,
          hardware: Arc<Hardware>,
        ) -> Result<(ServerDeviceIdentifier, Box<dyn ProtocolInitializer>), ButtplugDeviceError> {
          Ok((ServerDeviceIdentifier::new(hardware.address(), $protocol_identifier, &ProtocolAttributesType::Identifier(hardware.name().to_owned())), Box::new([< $protocol_name Initializer >]::default())))
        }
      }
    }
  };
}

pub use generic_protocol_initializer_setup;
pub use generic_protocol_setup;
