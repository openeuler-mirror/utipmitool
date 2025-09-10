/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#[allow(clippy::module_inception)]
pub mod sensor;

//use crate::commands::sdr::*;
use crate::commands::sensor::sensor::ipmi_sensor_list;
use crate::ipmi::intf::IpmiIntf;
use clap::{Parser, Subcommand};
use std::error::Error;

#[derive(Subcommand, Debug)]
pub enum SensorCommand {
    /// List all sensors and thresholds
    List,
    // /// Get detailed sensor information
    // Get {
    //     /// Sensor IDs (names)
    //     #[arg(required = true, num_args = 1..)]
    //     ids: Vec<String>
    // },

    // /// Manage sensor thresholds
    // Thresh(ThreshArgs),
}

#[derive(Parser)]
pub struct ThreshArgs {
    /// Sensor ID (name)
    pub id: String,

    #[command(subcommand)]
    pub subcmd: ThreshSubcommand,
}

#[derive(Subcommand)]
pub enum ThreshSubcommand {
    /// Set individual threshold
    Single {
        /// Threshold type
        #[arg(value_enum)]
        threshold: ThresholdType,

        /// Threshold value
        setting: f64,
    },

    /// Set all lower thresholds
    Lower {
        /// Values: lnr lcr lnc
        #[arg(required = true, num_args = 3)]
        values: Vec<f64>,
    },

    /// Set all upper thresholds
    Upper {
        /// Values: unc ucr unr
        #[arg(required = true, num_args = 3)]
        values: Vec<f64>,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum ThresholdType {
    /// Upper Non-Recoverable
    UNR,
    /// Upper Critical
    UCR,
    /// Upper Non-Critical
    UNC,
    /// Lower Non-Critical
    LNC,
    /// Lower Critical
    LCR,
    /// Lower Non-Recoverable
    LNR,
}

pub fn ipmi_sensor_main(
    command: SensorCommand,
    intf: Box<dyn IpmiIntf>,
) -> Result<(), Box<dyn Error>> {
    match command {
        SensorCommand::List => ipmi_sensor_list(intf),
        // SensorCommand::Get { ids } => ipmi_sensor_get(intf, &ids),
        // SensorCommand::Thresh(args) => {
        //     match args.subcmd {
        //         ThreshSubcommand::Single { threshold, setting } => {
        //             ipmi_sensor_set_threshold_single(intf, &args.id, threshold, setting)
        //         },
        //         ThreshSubcommand::Lower { values } => {
        //             ipmi_sensor_set_threshold_lower(intf, &args.id, &values)
        //         },
        //         ThreshSubcommand::Upper { values } => {
        //             ipmi_sensor_set_threshold_upper(intf, &args.id, &values)
        //         }
        //     }
        // }
    }
}
